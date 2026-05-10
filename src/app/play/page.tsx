'use client'

import { useEffect, useRef, useState } from 'react'
import Link from 'next/link'

// Matches music/src/instrument.rs
const RIGHT_BASE  = [261.63, 293.66, 329.63, 349.23, 392.00] // C4 D4 E4 F4 G4
const LEFT_BASE   = [246.94, 220.00, 196.00, 174.61, 164.81] // B3 A3 G3 F3 E3
const RIGHT_NAMES = ['C', 'D', 'E', 'F', 'G']
const LEFT_NAMES  = ['B', 'A', 'G', 'F', 'E']
// MediaPipe 21-landmark indices: thumb=0..4, index=5..8, middle=9..12, ring=13..16, pinky=17..20
const TIPS  = [4, 8, 12, 16, 20]
const BENDS = [3, 6, 10, 14, 18]
const AMPLITUDE = 0.22
const HARM_AMPS = [0.5, 1.0, 0.8, 0.4] // xylophone harmonics
const HARM_TOTAL = HARM_AMPS.reduce((a, b) => a + b, 0)

// Skeleton connections for drawing
const CONNECTIONS: [number, number][] = [
  [0,1],[1,2],[2,3],[3,4],
  [0,5],[5,6],[6,7],[7,8],
  [0,9],[9,10],[10,11],[11,12],
  [0,13],[13,14],[14,15],[15,16],
  [0,17],[17,18],[18,19],[19,20],
  [5,9],[9,13],[13,17],
]

interface Voice {
  oscillators: OscillatorNode[]
  envGain: GainNode
  active: boolean
}

function makeVoice(ctx: AudioContext, freq: number): Voice {
  const envGain = ctx.createGain()
  envGain.gain.setValueAtTime(0, ctx.currentTime)
  envGain.connect(ctx.destination)

  const oscillators = HARM_AMPS.map((amp, h) => {
    const osc = ctx.createOscillator()
    const harmGain = ctx.createGain()
    harmGain.gain.setValueAtTime(amp / HARM_TOTAL, ctx.currentTime)
    osc.type = 'sine'
    osc.frequency.setValueAtTime(freq * (h + 1), ctx.currentTime)
    osc.connect(harmGain)
    harmGain.connect(envGain)
    osc.start()
    return osc
  })

  return { oscillators, envGain, active: false }
}

function setFreq(ctx: AudioContext, voice: Voice, freq: number) {
  voice.oscillators.forEach((osc, h) => {
    osc.frequency.setValueAtTime(freq * (h + 1), ctx.currentTime)
  })
}

function noteOn(ctx: AudioContext, voice: Voice) {
  if (voice.active) return
  voice.active = true
  const now = ctx.currentTime
  voice.envGain.gain.cancelScheduledValues(now)
  voice.envGain.gain.setTargetAtTime(AMPLITUDE, now, 0.001) // fast attack
}

function noteOff(ctx: AudioContext, voice: Voice) {
  if (!voice.active) return
  voice.active = false
  const now = ctx.currentTime
  voice.envGain.gain.cancelScheduledValues(now)
  voice.envGain.gain.setTargetAtTime(0, now, 0.033) // ~200ms ring-out
}

type Status = 'idle' | 'loading' | 'ready' | 'error'

export default function PlayPage() {
  const videoRef = useRef<HTMLVideoElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const cleanupRef = useRef<(() => void) | null>(null)
  const activeNotesKeyRef = useRef('')
  const [status, setStatus] = useState<Status>('idle')
  const [activeNotes, setActiveNotes] = useState<string[]>([])

  useEffect(() => () => { cleanupRef.current?.() }, [])

  const start = async () => {
    setStatus('loading')
    try {
      // Dynamic import so MediaPipe only loads client-side on demand
      const { HandLandmarker, FilesetResolver } = await import('@mediapipe/tasks-vision')

      const vision = await FilesetResolver.forVisionTasks(
        'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.35/wasm'
      )
      const handLandmarker = await HandLandmarker.createFromOptions(vision, {
        baseOptions: {
          modelAssetPath:
            'https://storage.googleapis.com/mediapipe-models/hand_landmarker/hand_landmarker/float16/1/hand_landmarker.task',
          delegate: 'GPU',
        },
        runningMode: 'VIDEO',
        numHands: 2,
      })

      const audioCtx = new AudioContext()
      const voices = {
        right: RIGHT_BASE.map(f => makeVoice(audioCtx, f)),
        left:  LEFT_BASE.map(f => makeVoice(audioCtx, f)),
      }

      const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' } })
      const video = videoRef.current!
      video.srcObject = stream
      video.onloadedmetadata = () => {
        const canvas = canvasRef.current!
        canvas.width  = video.videoWidth
        canvas.height = video.videoHeight
      }
      await video.play()
      setStatus('ready')

      let rafId = 0
      const loop = () => {
        if (video.readyState >= 2 && video.videoWidth > 0) {
          const result = handLandmarker.detectForVideo(video, performance.now())
          const canvas = canvasRef.current!
          const ctx2d = canvas.getContext('2d')!
          ctx2d.clearRect(0, 0, canvas.width, canvas.height)

          // Midline: above = one octave higher
          ctx2d.save()
          ctx2d.setLineDash([6, 6])
          ctx2d.strokeStyle = 'rgba(148,163,184,0.5)'
          ctx2d.lineWidth = 1
          ctx2d.beginPath()
          ctx2d.moveTo(0, canvas.height / 2)
          ctx2d.lineTo(canvas.width, canvas.height / 2)
          ctx2d.stroke()
          ctx2d.restore()

          const W = canvas.width
          const H = canvas.height
          let sawRight = false
          let sawLeft  = false
          const notes: string[] = []

          result.landmarks.forEach((landmarks, i) => {
            // On raw (non-mirrored) front-camera: physical right hand → MediaPipe "Right"
            const handedness = result.handednesses[i]?.[0]?.categoryName
            const isRight = handedness === 'Right'
            if (isRight) sawRight = true; else sawLeft = true

            const wristY  = landmarks[0].y
            const octave  = wristY < 0.5 ? 2 : 1
            const voiceSet = isRight ? voices.right : voices.left
            const baseSet  = isRight ? RIGHT_BASE    : LEFT_BASE
            const nameSet  = isRight ? RIGHT_NAMES   : LEFT_NAMES

            TIPS.forEach((tipIdx, fi) => {
              const held = landmarks[tipIdx].y >= landmarks[BENDS[fi]].y
              setFreq(audioCtx, voiceSet[fi], baseSet[fi] * octave)
              if (held) {
                noteOn(audioCtx, voiceSet[fi])
                const sup = octave === 2 ? '′' : ''
                notes.push(nameSet[fi] + sup)
              } else {
                noteOff(audioCtx, voiceSet[fi])
              }
            })

            // Draw skeleton (natural coords — canvas has scaleX(-1) to match mirrored video)
            ctx2d.strokeStyle = 'rgba(148,163,184,0.7)'
            ctx2d.lineWidth = 1.5
            CONNECTIONS.forEach(([a, b]) => {
              ctx2d.beginPath()
              ctx2d.moveTo(landmarks[a].x * W, landmarks[a].y * H)
              ctx2d.lineTo(landmarks[b].x * W, landmarks[b].y * H)
              ctx2d.stroke()
            })

            landmarks.forEach((lm, idx) => {
              const fi = TIPS.indexOf(idx)
              const held = fi >= 0 && lm.y >= landmarks[BENDS[fi]].y
              const isTip = fi >= 0
              ctx2d.beginPath()
              ctx2d.arc(lm.x * W, lm.y * H, isTip ? 7 : 3, 0, Math.PI * 2)
              ctx2d.fillStyle = held ? '#ef4444' : isTip ? '#3b82f6' : '#cbd5e1'
              ctx2d.fill()
            })
          })

          if (!sawRight) voices.right.forEach(v => noteOff(audioCtx, v))
          if (!sawLeft)  voices.left.forEach(v => noteOff(audioCtx, v))

          const key = notes.join(',')
          if (key !== activeNotesKeyRef.current) {
            activeNotesKeyRef.current = key
            setActiveNotes([...notes])
          }
        }
        rafId = requestAnimationFrame(loop)
      }
      rafId = requestAnimationFrame(loop)

      cleanupRef.current = () => {
        cancelAnimationFrame(rafId)
        handLandmarker.close()
        audioCtx.close()
        stream.getTracks().forEach(t => t.stop())
      }
    } catch (e) {
      console.error(e)
      setStatus('error')
    }
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <Link href="/" className="text-blue-600 hover:underline mb-8 inline-block">
          ← back
        </Link>

        <h1 className="text-2xl font-normal mb-3">Play some music</h1>
        <p className="text-gray-600 mb-1 text-sm leading-relaxed">
          Right hand: thumb to pinky plays <span className="font-mono">C D E F G</span>.
          Left hand: thumb to pinky plays <span className="font-mono">B A G F E</span>.
        </p>
        <p className="text-gray-600 mb-6 text-sm">
          Curl a finger down to sound its note. Have fun!
        </p>

        {status === 'idle' && (
          <button
            onClick={start}
            className="px-4 py-2 bg-black text-white text-sm hover:bg-gray-800"
          >
            Start — needs camera access
          </button>
        )}
        {status === 'loading' && (
          <p className="text-gray-500 text-sm">Loading hand tracker…</p>
        )}
        {status === 'error' && (
          <p className="text-red-600 text-sm">
            Could not start. Check camera permissions and try again.
          </p>
        )}

        <div
          className="relative mt-4"
          style={{ display: status === 'ready' ? 'block' : 'none', maxWidth: 640 }}
        >
          <video
            ref={videoRef}
            playsInline
            muted
            style={{ width: '100%', display: 'block', transform: 'scaleX(-1)' }}
          />
          <canvas
            ref={canvasRef}
            width={640}
            height={480}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: '100%',
              transform: 'scaleX(-1)',
              pointerEvents: 'none',
            }}
          />
        </div>

        <div
          className="mt-4 font-mono text-xl tracking-widest text-gray-800"
          style={{ minHeight: '2rem' }}
        >
          {activeNotes.join('  ')}
        </div>
      </div>
    </div>
  )
}
