export interface Project {
  id: string
  title: string
  description?: string
  url: string
  date?: string
}

// You can add your GitHub projects here
const projects: Project[] = [
  {
    id: 'tostendout',
    title: 'tostendout',
    url: 'https://tostendout.com',
    date: '2024-12-19'
  },
  {
    id: 'cmdr',
    title: 'cmdr',
    url: 'https://github.com/vivienhenz24/cmdr',
    date: '2024-07-05'
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
