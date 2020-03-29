import { Dmg, WasmVideoOutput, WasmCameraSensor } from "dmg-frontend-web"
import Stats from "stats.js"

const stats = new Stats()
document.body.appendChild( stats.dom )

const display = document.getElementById("display")
const camera = document.getElementById("video")
const canvas = document.getElementById("canvas")

navigator.mediaDevices.getUserMedia({ video: { with: 128, height: 112 }, audio: false })
    .then(stream => {
        camera.srcObject = stream;
        camera.play()

        const video = WasmVideoOutput.with_context(display.getContext("2d"));
        const sensor = WasmCameraSensor.with_video_and_context(camera, canvas.getContext("2d"))
        const dmg = Dmg.with_video_and_sensor(video, sensor)

        document.addEventListener("keydown", (event) => dmg.handle_key_down(event))
        document.addEventListener("keyup", (event) => dmg.handle_key_up(event))

        const update = () => {
            stats.begin()
            dmg.emulate_frame()
            stats.end()
            window.requestAnimationFrame(update)
        }

        window.requestAnimationFrame(update)
    })

