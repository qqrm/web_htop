import { h, render } from 'https://unpkg.com/preact@latest?module';
import htm from 'https://unpkg.com/htm?module';


const html = htm.bind(h);

function App(props) {
    return html`
        <h1>Cpu load</h1>
        <div>
        ${props.cpus.map((cpu) => {
        return html`<div class="bar">
            <div class="bar-inner" style="width: ${cpu}%"></div>
            <label>${cpu.toFixed(2)}% usage</label>
        </div>`;
    })}
    </div>`;
}


let url = new URL("/rt/cpus", window.location.href);
url.protocol = url.protocol.replace("http", "ws");

let ws = new WebSocket(url.href);
ws.onmessage = (ev) => {
    let json = JSON.parse(ev.data);
    render(html`<${App} cpus=${json}></${App}>`, document.body);
};