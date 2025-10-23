import { getAllEssays } from './essays'
import { getAllProjects } from './projects'

export interface ContentItem {
  id: string
  title: string
  date: string
  type: 'essay' | 'project'
  htmlContent?: string
  originalDate: string
}

// Get all content (essays + projects) sorted by date
export function getAllContent(): ContentItem[] {
  const essays = getAllEssays()
  const projects = getAllProjects()
  
  // Both essays and projects already have the same structure
  const allContent = [
    ...essays.map(e => ({ ...e, type: 'essay' as const })),
    ...projects.map(p => ({ ...p, type: 'project' as const }))
  ]
  
  return allContent.sort((a, b) => 
    new Date(b.originalDate).getTime() - new Date(a.originalDate).getTime()
  )
}

// Get content by ID (searches both essays and projects)
export function getContentById(id: string): ContentItem | null {
  const essays = getAllEssays()
  const projects = getAllProjects()
  
  const essay = essays.find(e => e.id === id)
  if (essay) return { ...essay, type: 'essay' as const }
  
  const project = projects.find(p => p.id === id)
  if (project) return { ...project, type: 'project' as const }
  
  return null
}
