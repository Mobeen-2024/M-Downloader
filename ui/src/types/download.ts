export type DownloadStatus = 'Downloading' | 'Paused' | 'Completed' | 'Error' | 'Queued' | 'RefreshNeeded';

export interface SegmentInfo {
  start: number;
  end: number;
  downloaded: number;
  state: 'Pending' | 'Active' | 'Completed' | 'Failed';
  retry_count: number;
  last_latency_ms: number;
}

export interface EngineEventStats {
  total_splits: number;
  total_retries: number;
  http_version: string;
}

export interface DownloadMetrics {
  io_efficiency: number;
  active_workers: number;
  avg_latency_ms: number;
  engine_stats?: EngineEventStats;
}

export interface DownloadProgressEvent {
  id: string;
  downloaded: number;
  total: number;
  speed_bps: number;
  status: DownloadStatus;
  segments: SegmentInfo[];
  last_error_code?: number;
  metrics?: DownloadMetrics;
}

export interface DownloadItem extends DownloadProgressEvent {
  name: string;
  url: string;
  addedAt: number;
  category?: string;
}
