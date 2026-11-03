import { render } from 'preact';

import { SettingsWindow } from './SettingsWindow';

import './settings.css';

const root = document.querySelector('#root');
if (root === null) {
  throw new Error('settings.html has no #root element');
}

render(<SettingsWindow />, root);
