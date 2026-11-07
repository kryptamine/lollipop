import { render } from 'preact';

import { RecapWindow } from './RecapWindow';

import './settings.css';
import './recap.css';

const root = document.querySelector('#root');
if (root === null) {
  throw new Error('recap.html has no #root element');
}

render(<RecapWindow />, root);
