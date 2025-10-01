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
    date: '2025-01-01'
  },
  {
    id: 'parsed',
    title: 'Parsed',
    description: 'A mobile application available on the App Store that helps users parse and organize information in a clean, intuitive interface.',
    url: 'https://apps.apple.com/tn/app/parsed/id6743483636',
    date: '2024-09-01'
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
