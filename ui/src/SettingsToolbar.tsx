export type SettingsTab = 'general' | 'overlay';

type ToolbarItem = {
  tab: SettingsTab;
  icon: string;
  label: string;
};

const ITEMS: ToolbarItem[] = [
  {
    tab: 'general',
    icon: '⚙︎',
    label: 'General',
  },
  {
    tab: 'overlay',
    icon: '▣',
    label: 'Overlay',
  },
];

type SettingsToolbarProps = {
  shown: SettingsTab;
  onSelect: (tab: SettingsTab) => void;
};

export const SettingsToolbar = ({ shown, onSelect }: SettingsToolbarProps) => (
  <div class="toolbar" role="tablist">
    {ITEMS.map(({ tab, icon, label }) => (
      <button
        key={tab}
        type="button"
        role="tab"
        aria-selected={tab === shown}
        onClick={() => {
          onSelect(tab);
        }}
      >
        <span class="icon" aria-hidden="true">
          {icon}
        </span>
        {label}
      </button>
    ))}
  </div>
);
