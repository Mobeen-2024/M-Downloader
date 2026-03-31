export function useFormatters() {
  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatSpeed = (bps: number): string => {
    return formatSize(bps) + '/s';
  };

  const formatEta = (downloaded: number, total: number, bps: number): string => {
    if (bps === 0) return '--';
    const remaining = total - downloaded;
    const seconds = Math.floor(remaining / bps);
    
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
    
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m`;
  };

  return { formatSize, formatSpeed, formatEta };
}
