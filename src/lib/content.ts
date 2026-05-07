import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { remark } from 'remark'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import remarkRehype from 'remark-rehype'
import rehypeRaw from 'rehype-raw'
import rehypeKatex from 'rehype-katex'
import rehypeStringify from 'rehype-stringify'

export interface ContentItem {
  id: string
  title: string
  date: string
  type: 'essay' | 'project'
  htmlContent: string
  originalDate: string
  url?: string
  description?: string
}

const EXCLUDED_ESSAYS = ['orwell', 'betweenthelines', 'denial']

function readDir(dir: string, type: 'essay' | 'project', exclude: string[] = []): ContentItem[] {
  return fs.readdirSync(dir)
    .filter(f => f.endsWith('.md'))
    .map(fileName => {
      const id = fileName.replace(/\.md$/, '')
      const { data, content } = matter(fs.readFileSync(path.join(dir, fileName), 'utf8'))
      const htmlContent = remark()
        .use(remarkGfm)
        .use(remarkMath)
        .use(remarkRehype, { allowDangerousHtml: true })
        .use(rehypeRaw)
        .use(rehypeKatex)
        .use(rehypeStringify)
        .processSync(content)
        .toString()
      const originalDate = new Date(data.date + 'T00:00:00')
      return {
        id,
        title: data.title,
        date: originalDate.toLocaleDateString('en-US', { year: 'numeric', month: 'long' }),
        type,
        htmlContent,
        originalDate: originalDate.toISOString(),
        ...(data.url && { url: data.url }),
        ...(data.description && { description: data.description }),
      }
    })
    .filter(item => !exclude.includes(item.id))
}

export function getAllContent(): ContentItem[] {
  const essays = readDir(path.join(process.cwd(), 'src/content/essays'), 'essay', EXCLUDED_ESSAYS)
  const projects = readDir(path.join(process.cwd(), 'src/content/projects'), 'project')
  return [...essays, ...projects].sort((a, b) =>
    new Date(b.originalDate).getTime() - new Date(a.originalDate).getTime()
  )
}

export function getContentById(id: string): ContentItem | null {
  return getAllContent().find(item => item.id === id) ?? null
}
