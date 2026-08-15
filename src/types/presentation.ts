import type { DevicePlatform, PresenceState, TransferState } from "./index";

export type CollectionSource = "bridge" | "development-fixture" | "empty";
export type CollectionSyncState = "idle" | "loading" | "ready" | "stale" | "offline" | "error";

export interface DeviceListItem {
  id: string;
  name: string;
  platform: DevicePlatform;
  presence: PresenceState;
  lastSeen: string;
}

export type TransferListStatus = "queued" | "active" | "complete";

export interface TransferListItem {
  id: string;
  name: string;
  type: string;
  status: TransferListStatus;
  progress: number;
  deviceName: string;
  updatedAt: string;
  domainState?: TransferState;
}
