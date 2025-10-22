export interface Project {
  id: string
  title: string
  description?: string
  content?: string
  url: string
  date?: string
}

// You can add your GitHub projects here
const projects: Project[] = [
  {
    id: 'transformer-candle',
    title: 'Building a transformer from scratch',
    description: 'A complete implementation of a transformer neural network architecture built entirely from scratch using Rust and Candle. Features RoPE (Rotary Position Embedding) and BBPE (Byte-level BPE tokenization), with full attention mechanisms, positional encoding, and multi-head attention.<br><br>There\'s no rational reason for building a transformer in Rust—except that it\'s fun.<br><br><a href="https://github.com/vivienhenz24/transformer_candle" target="_blank" rel="noopener noreferrer" style="color: #2563eb; text-decoration: underline;">View on GitHub →</a>',
    url: 'https://github.com/vivienhenz24/transformer_candle',
    date: '2025-09-01'
  },
  {
    id: 'parsed',
    title: 'Parsed',
    description: 'My first big coding project, a mobile app that gives you the five most important news headlines of the day, articles about these headlines written by newspapers from all over the political spectrum. And a bias analysis for each news item presented to you. 2500+ downloads as of today :) Availaible on the app store.<br><br><a href="https://apps.apple.com/tn/app/parsed/id6743483636" target="_blank" rel="noopener noreferrer" style="color: #2563eb; text-decoration: underline;">Download on the App Store →</a>',
    url: 'https://apps.apple.com/tn/app/parsed/id6743483636',
    date: '2025-03-29'
  },
  {
    id: 'audio-steganography',
    title: 'Hiding a secret message inside an audio file',
    description: '',
    url: '/audio-steganography',
    date: '2025-10-17'
  },
  {
    id: 'luxembourgish-tts',
    title: 'Luxembourgish text-to-speech',
    description: 'Available at <a href="https://neiom.io" target="_blank" rel="noopener noreferrer" style="color: #2563eb; text-decoration: underline;">neiom.io</a>',
    url: '/luxembourgish-tts',
    date: '2025-10-17'
  }
  // Add more projects here...
]

export function getAllProjects(): Project[] {
  return projects.sort((a, b) => {
    if (!a.date || !b.date) return 0
    return new Date(b.date).getTime() - new Date(a.date).getTime()
  })
}

export function getProjectById(id: string): Project | null {
  return projects.find(project => project.id === id) || null
}
