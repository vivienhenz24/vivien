import fs from 'fs'
import path from 'path'

// Path to pre-processed essays data
const essaysDataPath = path.join(process.cwd(), 'src/lib/essays-data.json')

// Cache for runtime (minimal CPU usage)
let essaysData: Essay[] | null = null

export interface Essay {
  id: string
  title: string
  date: string
  content: string
  htmlContent: string
  originalDate: string // ISO string for JSON compatibility
}

// Load essays data once (zero CPU processing at runtime)
function loadEssaysData(): Essay[] {
  if (essaysData === null) {
    try {
      const data = fs.readFileSync(essaysDataPath, 'utf8')
      essaysData = JSON.parse(data)
    } catch (error) {
      console.error('Failed to load essays data:', error)
      essaysData = []
    }
  }
  return essaysData || []
}

export function getAllEssayIds(): string[] {
  const essays = loadEssaysData()
  return essays.map(essay => essay.id)
}

export function getEssayById(id: string): Essay | null {
  const essays = loadEssaysData()
  return essays.find(essay => essay.id === id) || null
}

export function getAllEssays(): Essay[] {
  return loadEssaysData()
} 