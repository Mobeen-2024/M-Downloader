export type DownloadStatus = 'Downloading' | 'Paused' | 'Completed' | 'Error' | 'Queued';

export interface SegmentInfo {
  start: number;
  end: number;
  downloaded: number;
  active: boolean;
}

export interface DownloadProgressEvent {
  id: string;
  downloaded: number;
  total: number;
  speed_bps: number;
  status: DownloadStatus;
  segments: SegmentInfo[];
}

export interface DownloadItem extends DownloadProgressEvent {
  name: string;
  url: string;
  addedAt: number;
}
