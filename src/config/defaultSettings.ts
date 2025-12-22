export interface GeneralSettings {
  authorName: string;
  dailyGoal: number;
  autoSaveInterval: number; // in seconds
}

export interface EditorSettings {
  fontFamily: 'sans' | 'serif' | 'mono';
  fontSize: number;
  lineHeight: number;
  maxWidth: number; // in px, for the editor container
  focusMode: boolean; // highlight active paragraph only
}

export interface InterfaceSettings {
  theme: 'light' | 'dark' | 'system';
  cyberGlassIntensity: number; // 0-100 scale for blur/transparency effects
  uiScaling: number; // 80 to 120 percentage
}

export interface AppSettings {
  general: GeneralSettings;
  editor: EditorSettings;
  interface: InterfaceSettings;
}

export const defaultSettings: AppSettings = {
  general: {
    authorName: '',
    dailyGoal: 500,
    autoSaveInterval: 30,
  },
  editor: {
    fontFamily: 'serif',
    fontSize: 18,
    lineHeight: 1.6,
    maxWidth: 800,
    focusMode: false,
  },
  interface: {
    theme: 'light',
    cyberGlassIntensity: 50,
    uiScaling: 100,
  },
};
