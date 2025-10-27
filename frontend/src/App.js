import React, { useState, useCallback } from 'react';
import './App.css';
import { FFprobeWorker } from 'ffprobe-wasm';

function App() {
  const [file, setFile] = useState(null);
  const [outputFormat, setOutputFormat] = useState('mp3');
  const [compression, setCompression] = useState(128);
  const [estimatedSize, setEstimatedSize] = useState(null);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [downloadLink, setDownloadLink] = useState(null);
  const [duration, setDuration] = useState(null);

  const calculateEstimatedSize = useCallback(
    (videoDuration) => {
      if (videoDuration) {
        const estimated = (videoDuration * compression * 1000) / 8 / 1024; // in KB
        setEstimatedSize(estimated.toFixed(2));
      } else {
        setEstimatedSize(null);
      }
    },
    [compression]
  );

  const handleFileChange = async (event) => {
    const selectedFile = event.target.files[0];
    setFile(selectedFile);

    if (selectedFile) {
      const worker = new FFprobeWorker();
      const metadata = await worker.getFileInfo(selectedFile);
      const videoDuration = metadata.streams[0].duration;
      setDuration(videoDuration);
      calculateEstimatedSize(videoDuration);
    } else {
      setEstimatedSize(null);
    }
  };

  const handleFormatChange = (event) => {
    setOutputFormat(event.target.value);
  };

  const handleCompressionChange = (event) => {
    setCompression(event.target.value);
    calculateEstimatedSize(duration);
  };

  const handleSubmit = async (event) => {
    event.preventDefault();

    if (!file) {
      return;
    }

    const formData = new FormData();
    formData.append('file', file);
    formData.append('format', outputFormat);
    formData.append('compression', compression);

    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener('progress', (event) => {
      if (event.lengthComputable) {
        const percentComplete = (event.loaded / event.total) * 100;
        setUploadProgress(percentComplete);
      }
    });

    xhr.addEventListener('load', () => {
      const response = JSON.parse(xhr.responseText);
      setDownloadLink(response.downloadUrl);
    });

    xhr.open('POST', '/upload');
    xhr.send(formData);
  };

  return (
    <div className="App">
      <div className="card">
        <div className="card-header">
          <h1>Audio Extractor</h1>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="file-upload">Select a video file:</label>
            <input id="file-upload" type="file" onChange={handleFileChange} />
          </div>
          <div className="form-group">
            <label htmlFor="format-select">Output format:</label>
            <select id="format-select" value={outputFormat} onChange={handleFormatChange}>
              <option value="mp3">MP3</option>
              <option value="wav">WAV</option>
              <option value="aac">AAC</option>
            </select>
          </div>
          <div className="form-group">
            <label htmlFor="compression-slider">Compression (kbps):</label>
            <input
              id="compression-slider"
              type="range"
              min="64"
              max="320"
              step="32"
              value={compression}
              onChange={handleCompressionChange}
            />
            <span>{compression} kbps</span>
          </div>
          {estimatedSize && <p>Estimated file size: {estimatedSize} KB</p>}
          <button type="submit">Extract Audio</button>
        </form>
        {uploadProgress > 0 && <progress value={uploadProgress} max="100" />}
        {downloadLink && <a href={downloadLink}>Download Extracted Audio</a>}
      </div>
    </div>
  );
}

export default App;
