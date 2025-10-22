import { getAllEssays, Essay } from './essays'
import { getAllProjects, Project } from './projects'

export interface ContentItem {
  id: string
  title: string
  date: string
  type: 'essay' | 'project'
  content?: string
  htmlContent?: string
  description?: string
  url?: string
  originalDate: string // ISO string for sorting
}

// Convert essays to unified content items
function essaysToContentItems(essays: Essay[]): ContentItem[] {
  return essays.map(essay => ({
    id: essay.id,
    title: essay.title,
    date: essay.date,
    type: 'essay' as const,
    content: essay.content,
    htmlContent: essay.htmlContent,
    originalDate: essay.originalDate
  }))
}

// Convert projects to unified content items
function projectsToContentItems(projects: Project[]): ContentItem[] {
  return projects.map(project => ({
    id: project.id,
    title: project.title,
    date: project.date || '',
    type: 'project' as const,
    description: project.description,
    url: project.url,
    originalDate: project.date ? new Date(project.date).toISOString() : new Date().toISOString()
  }))
}

// Get all content items (essays + projects) sorted by date
export function getAllContent(): ContentItem[] {
  const essays = getAllEssays()
  const projects = getAllProjects()
  
  const essayItems = essaysToContentItems(essays)
  const projectItems = projectsToContentItems(projects)
  
  const allContent = [...essayItems, ...projectItems]
  
  return allContent.sort((a, b) => {
    return new Date(b.originalDate).getTime() - new Date(a.originalDate).getTime()
  })
}

// Get content item by ID (searches both essays and projects)
export function getContentById(id: string): ContentItem | null {
  const essays = getAllEssays()
  const projects = getAllProjects()
  
  // Check essays first
  const essay = essays.find(e => e.id === id)
  if (essay) {
    return essaysToContentItems([essay])[0]
  }
  
  // Check projects
  const project = projects.find(p => p.id === id)
  if (project) {
    return projectsToContentItems([project])[0]
  }
  
  return null
}

// Get all content IDs
export function getAllContentIds(): string[] {
  const content = getAllContent()
  return content.map(item => item.id)
}

// Legacy functions for backward compatibility
export { getAllEssays, getEssayById, getAllEssayIds } from './essays'
export { getAllProjects, getProjectById } from './projects'
