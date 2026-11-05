import { invoke } from '@tauri-apps/api/core';

export interface DriveInfo {
  drive: string;
  mountPoint: string;
  label: string;
  driveType: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  totalGb: number;
  freeGb: number;
  usedPercentage: number;
  isSystem: boolean;
  isReadOnly: boolean;
  isLowSpace: boolean;
  fileSystem?: string;
}

export interface DirectoryListing {
  currentPath: string;
  parentPath: string | null;
  canWrite: boolean;
  entries: DirectoryEntryItem[];
  totalFolders: number;
  freeSpaceBytes: number;
  freeSpaceGb: number;
}

export interface DirectoryEntryItem {
  name: string;
  path: string;
  isDirectory: boolean;
  isHidden: boolean;
  isSystem: boolean;
  isAccessible: boolean;
  modifiedTime?: number;
}

export async function fetchSystemDrives(): Promise<DriveInfo[]> {
  try {
    return await invoke<DriveInfo[]>('get_system_drives');
  } catch (err) {
    console.error('Failed to get system drives:', err);
    return [
      {
        drive: 'C:',
        mountPoint: 'C:\\',
        label: 'Local Disk (C:)',
        driveType: 'Fixed',
        totalBytes: 500000000000,
        freeBytes: 150000000000,
        usedBytes: 350000000000,
        totalGb: 465.6,
        freeGb: 139.7,
        usedPercentage: 70,
        isSystem: true,
        isReadOnly: false,
        isLowSpace: false,
        fileSystem: 'NTFS',
      },
    ];
  }
}

export async function listDirectory(path: string): Promise<DirectoryListing> {
  return await invoke<DirectoryListing>('list_directory_contents', { path });
}

export async function createNewFolder(path: string): Promise<void> {
  await invoke('create_directory', { path });
}

export function calculateUtilization(total: number, free: number): number {
  if (total <= 0) return 0;
  const used = total - free;
  return Math.round((used / total) * 100);
}

export function isValidWindowsInstallPath(p: string): boolean {
  if (p.includes('..')) return false;
  return /^[a-zA-Z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/.test(p);
}

export function canInstallGame(driveFreeBytes: number, gameSizeBytes: number, safetyMarginBytes = 1073741824): boolean {
  return driveFreeBytes >= gameSizeBytes + safetyMarginBytes;
}

export function isReservedWindowsName(folderName: string): boolean {
  const reserved = new Set([
    'CON', 'PRN', 'AUX', 'NUL',
    'COM0', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
    'LPT0', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
  ]);
  const trimmed = folderName.trim().replace(/\.+$/, '');
  const stem = trimmed.split('.')[0]?.trim().toUpperCase() || '';
  return reserved.has(stem);
}

class DirectoryPickerStore {
  isOpen = $state<boolean>(false);
  currentPath = $state<string>('');
  drives = $state<DriveInfo[]>([]);
  listing = $state<DirectoryListing | null>(null);
  loading = $state<boolean>(false);
  title = $state<string>('Select Installation Directory');
  requiredBytes = $state<number>(0);
  private resolveCallback: ((path: string | null) => void) | null = null;

  async open(title = 'Select Directory', initialPath?: string, requiredBytes = 0): Promise<string | null> {
    this.title = title;
    this.requiredBytes = requiredBytes;
    this.isOpen = true;
    this.loading = true;

    try {
      this.drives = await fetchSystemDrives();
      const target = initialPath || (this.drives[0] ? this.drives[0].mountPoint : 'C:\\');
      await this.navigateTo(target);
    } finally {
      this.loading = false;
    }

    return new Promise<string | null>((resolve) => {
      this.resolveCallback = resolve;
    });
  }

  async navigateTo(path: string) {
    this.loading = true;
    try {
      const res = await listDirectory(path);
      this.currentPath = res.currentPath;
      this.listing = res;
    } catch (err) {
      console.error(`Failed to navigate to ${path}:`, err);
    } finally {
      this.loading = false;
    }
  }

  async createFolder(name: string) {
    if (!this.currentPath || !name.trim()) return;
    if (isReservedWindowsName(name)) {
      throw new Error(`'${name}' is a reserved Windows device name`);
    }
    const cleanPath = this.currentPath.endsWith('\\')
      ? `${this.currentPath}${name.trim()}`
      : `${this.currentPath}\\${name.trim()}`;

    await createNewFolder(cleanPath);
    await this.navigateTo(cleanPath);
  }

  select(path: string) {
    this.isOpen = false;
    if (this.resolveCallback) {
      this.resolveCallback(path);
      this.resolveCallback = null;
    }
  }

  cancel() {
    this.isOpen = false;
    if (this.resolveCallback) {
      this.resolveCallback(null);
      this.resolveCallback = null;
    }
  }
}

export const directoryPicker = new DirectoryPickerStore();
