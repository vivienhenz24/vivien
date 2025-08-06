import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { remark } from 'remark'
import html from 'remark-html'

const essaysDirectory = path.join(process.cwd(), 'src/content/essays')

export interface Essay {
  id: string
  title: string
  date: string
  content: string
  htmlContent: string
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
      .use(html)
      .processSync(matterResult.content)
    const contentHtml = processedContent.toString()
    
    // Format date as "Month Year"
    const date = new Date(matterResult.data.date)
    const formattedDate = date.toLocaleDateString('en-US', { 
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
    }
  } catch (error) {
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
    
    // Format date as "Month Year"
    const date = new Date(matterResult.data.date)
    const formattedDate = date.toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long' 
    })
    
    // Combine the data with the id
    return {
      id,
      title: matterResult.data.title,
      date: formattedDate,
      content: matterResult.content,
      htmlContent: '', // We don't need HTML for the list view
    }
  })
  
  // Sort essays by date
  return allEssaysData.sort((a, b) => {
    if (a.date < b.date) {
      return 1
    } else {
      return -1
    }
  })
} 