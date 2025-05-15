import './index.css';
import { localize } from '@private/monaco-editor/localize';

await localize();
await import('./index.js');
