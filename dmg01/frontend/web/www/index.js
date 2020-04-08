import { Dmg, WasmVideoOutput, init_wasm } from "dmg-frontend-web"

Error.stackTraceLimit = 100;
init_wasm()

const display = document.getElementById("display")
const video = WasmVideoOutput.new(display.getContext("2d"));
const dmg = Dmg.new(video)

document.addEventListener("keydown", (event) => dmg.handle_key_down(event))
document.addEventListener("keyup", (event) => dmg.handle_key_up(event))

const update = () => {
    dmg.emulate_frame()
    window.requestAnimationFrame(update)
}

window.requestAnimationFrame(update)
