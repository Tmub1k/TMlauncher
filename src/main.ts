import { mount } from 'svelte';
import App from './App.svelte';
import { registerOmniWidgetElements } from './lib/widgets/widget-host';

registerOmniWidgetElements();

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
