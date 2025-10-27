use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use ffmpeg_next as ffmpeg;
use futures_util::TryStreamExt;
use sanitize_filename::sanitize;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use actix_web_static_files;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Serialize)]
struct UploadResponse {
    download_url: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

fn extract_audio(
    input_path: &Path,
    output_path: &Path,
    format: &str,
    bitrate: i32,
) -> Result<(), Error> {
    ffmpeg::init().map_err(Error::from)?;
    let mut ictx = ffmpeg::format::input(&input_path).map_err(Error::from)?;
    let mut octx = ffmpeg::format::output(&output_path).map_err(Error::from)?;

    let best_audio_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or_else(|| ffmpeg::Error::StreamNotFound)
        .map_err(Error::from)?;
    let best_audio_stream_index = best_audio_stream.index();

    let mut decoder = best_audio_stream.codec().decoder().audio().map_err(Error::from)?;
    let codec = match format {
        "mp3" => ffmpeg::codec::Id::MP3,
        "wav" => ffmpeg::codec::Id::PCM_S16LE,
        "aac" => ffmpeg::codec::Id::AAC,
        _ => ffmpeg::codec::Id::MP3,
    };

    let mut ost = octx.add_stream(ffmpeg::encoder::find(codec)).map_err(Error::from)?;
    let mut encoder = ost.codec().encoder().audio().map_err(Error::from)?;
    encoder.set_bit_rate(bitrate * 1000);
    encoder.set_sample_rate(decoder.sample_rate());
    encoder.set_channel_layout(decoder.channel_layout());
    encoder.set_format(decoder.format());

    octx.write_header().map_err(Error::from)?;

    let mut decoded = ffmpeg::frame::Audio::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == best_audio_stream_index {
            decoder.send_packet(&packet).map_err(Error::from)?;
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut encoded = ffmpeg::Packet::empty();
                encoder.send_frame(&decoded).map_err(Error::from)?;
                while encoder.receive_packet(&mut encoded).is_ok() {
                    encoded.write_interleaved(&mut octx).map_err(Error::from)?;
                }
            }
        }
    }
    octx.write_trailer().map_err(Error::from)?;
    Ok(())
}

#[post("/upload")]
async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut filename = String::new();
    let mut file_data: Vec<u8> = Vec::new();
    let mut format = String::from("mp3");
    let mut compression = 128;

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = content_disposition
                .get_filename()
                .map_or_else(|| "unknown".to_string(), |f| f.to_string());
            while let Some(chunk) = field.try_next().await? {
                file_data.extend_from_slice(&chunk);
            }
        } else if field_name == "format" {
            while let Some(chunk) = field.try_next().await? {
                format = String::from_utf8(chunk.to_vec()).unwrap_or_default();
            }
        } else if field_name == "compression" {
            while let Some(chunk) = field.try_next().await? {
                compression = String::from_utf8(chunk.to_vec())
                    .unwrap_or_default()
                    .parse::<i32>()
                    .unwrap_or(128);
            }
        }
    }

    let sanitized_filename = sanitize(&filename);
    let temp_dir = std::env::temp_dir().join("audio_extractor");
    std::fs::create_dir_all(&temp_dir)?;
    let unique_filename = format!("{}-{}", Uuid::new_v4(), sanitized_filename);
    let input_filepath = temp_dir.join(&unique_filename);

    web::block(move || std::fs::write(&input_filepath, file_data)).await??;

    let output_filename = format!(
        "{}.{}",
        Path::new(&sanitized_filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap(),
        format
    );
    let output_filepath = PathBuf::from("./output").join(&output_filename);
    std::fs::create_dir_all("./output")?;

    web::block(move || {
        extract_audio(
            &temp_dir.join(&unique_filename),
            &output_filepath,
            &format,
            compression,
        )
    })
    .await??;

    let response = UploadResponse {
        download_url: format!("/output/{}", output_filename),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let generated = generate();
        App::new()
            .service(hello)
            .service(save_file)
            .service(Files::new("/output", "./output"))
            .service(actix_web_static_files::ResourceFiles::new("/", generated))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
