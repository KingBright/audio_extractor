package com.example.videoaudioextractor;

import android.content.Context;
import android.net.Uri;
import android.util.Log;
import com.arthenica.ffmpegkit.FFmpegKit;
import com.arthenica.ffmpegkit.ReturnCode;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

public class AudioExtractor {

    public interface Callback {
        void onSuccess(String outputPath);
        void onFailure(String error);
    }

    public static void extractAudio(Context context, Uri videoUri, Callback callback) {
        File inputFile = createCacheFileFromUri(context, videoUri);
        if (inputFile == null) {
            callback.onFailure("Failed to create a temporary file from the video URI.");
            return;
        }

        File outputDir = new File(context.getExternalCacheDir(), "audio");
        if (!outputDir.exists()) {
            outputDir.mkdirs();
        }
        String outputPath = new File(outputDir, "extracted_audio.aac").getAbsolutePath();

        String command = "-y -i \"" + inputFile.getAbsolutePath() + "\" -vn -acodec copy \"" + outputPath + "\"";

        FFmpegKit.executeAsync(command, session -> {
            // Clean up the temporary input file
            inputFile.delete();

            if (ReturnCode.isSuccess(session.getReturnCode())) {
                callback.onSuccess(outputPath);
            } else {
                Log.e("AudioExtractor", "FFmpeg execution failed: " + session.getFailStackTrace());
                callback.onFailure("FFmpeg execution failed. See logs for details.");
            }
        });
    }

    private static File createCacheFileFromUri(Context context, Uri uri) {
        try {
            InputStream inputStream = context.getContentResolver().openInputStream(uri);
            if (inputStream == null) return null;

            File tempFile = new File(context.getCacheDir(), "temp_video_file");
            try (OutputStream outputStream = new FileOutputStream(tempFile)) {
                byte[] buffer = new byte[4096];
                int bytesRead;
                while ((bytesRead = inputStream.read(buffer)) != -1) {
                    outputStream.write(buffer, 0, bytesRead);
                }
            }
            inputStream.close();
            return tempFile;
        } catch (Exception e) {
            Log.e("AudioExtractor", "Failed to create cache file from URI", e);
            return null;
        }
    }
}
