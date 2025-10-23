import fs from 'fs'
import path from 'path'

// Path to pre-processed projects data
const projectsDataPath = path.join(process.cwd(), 'src/lib/projects-data.json')

// Cache for runtime (minimal CPU usage)
let projectsData: Project[] | null = null

export interface Project {
  id: string
  title: string
  description?: string
  content?: string
  htmlContent?: string
  url: string
  date: string
  originalDate: string // ISO string for JSON compatibility
}

// Load projects data once (zero CPU processing at runtime)
function loadProjectsData(): Project[] {
  if (projectsData === null) {
    try {
      const data = fs.readFileSync(projectsDataPath, 'utf8')
      projectsData = JSON.parse(data)
    } catch (error) {
      console.error('Failed to load projects data:', error)
      projectsData = []
    }
  }
  return projectsData || []
}

export function getAllProjectIds(): string[] {
  const projects = loadProjectsData()
  return projects.map(project => project.id)
}

export function getProjectById(id: string): Project | null {
  const projects = loadProjectsData()
  return projects.find(project => project.id === id) || null
}

export function getAllProjects(): Project[] {
  return loadProjectsData()
}
