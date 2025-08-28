import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { remark } from 'remark'
import html from 'remark-html'
import remarkGfm from 'remark-gfm'

const essaysDirectory = path.join(process.cwd(), 'src/content/essays')

export interface Essay {
  id: string
  title: string
  date: string
  content: string
  htmlContent: string
  originalDate: Date
}

export function getAllEssayIds(): string[] {
  const fileNames = fs.readdirSync(essaysDirectory)
  return fileNames.map(fileName => {
    return fileName.replace(/\.md$/, '')
  })
}

export function getEssayById(id: string): Essay | null {
  try {
    const fullPath = path.join(essaysDirectory, `${id}.md`)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    
    // Use gray-matter to parse the post metadata section
    const matterResult = matter(fileContents)
    
    // Use remark to convert markdown into HTML string
    const processedContent = remark()
      .use(remarkGfm)
      .use(html)
      .processSync(matterResult.content)
    const contentHtml = processedContent.toString()
    
    // Parse the original date
    const originalDate = new Date(matterResult.data.date)
    
    // Format date as "Month Year"
    const formattedDate = originalDate.toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long' 
    })
    
    // Combine the data with the id
    return {
      id,
      title: matterResult.data.title,
      date: formattedDate,
      content: matterResult.content,
      htmlContent: contentHtml,
      originalDate: originalDate,
    }
  } catch {
    return null
  }
}

export function getAllEssays(): Essay[] {
  const fileNames = fs.readdirSync(essaysDirectory)
  const allEssaysData = fileNames.map((fileName) => {
    // Remove ".md" from file name to get id
    const id = fileName.replace(/\.md$/, '')
    
    // Read markdown file as string
    const fullPath = path.join(essaysDirectory, fileName)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    
    // Use gray-matter to parse the post metadata section
    const matterResult = matter(fileContents)
    
    // Use remark to convert markdown into HTML string
    const processedContent = remark()
      .use(remarkGfm)
      .use(html)
      .processSync(matterResult.content)
    const contentHtml = processedContent.toString()
    
    // Parse the original date for sorting
    const originalDate = new Date(matterResult.data.date)
    
    // Format date as "Month Year" for display
    const formattedDate = originalDate.toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long' 
    })
    
    // Combine the data with the id
    return {
      id,
      title: matterResult.data.title,
      date: formattedDate,
      content: matterResult.content,
      htmlContent: contentHtml,
      // Store original date for sorting
      originalDate: originalDate,
    }
  })
  
  // Sort essays by date (newest first)
  return allEssaysData.sort((a, b) => {
    return b.originalDate.getTime() - a.originalDate.getTime()
  })
} 