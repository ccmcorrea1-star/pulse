export type DevicePlatform = "linux" | "android" | "ios" | "windows";

export interface Device {
  id: string;
  name: string;
  platform: DevicePlatform;
  online: boolean;
  lastSeen: string;
}

export type TransferStatus = "queued" | "in-progress" | "complete";

export interface Transfer {
  id: string;
  name: string;
  type: string;
  status: TransferStatus;
  progress: number;
  deviceName: string;
  updatedAt: string;
}

export type BridgeState = "idle" | "loading" | "success" | "error";
