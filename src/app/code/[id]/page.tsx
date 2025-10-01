import Link from 'next/link'
import { notFound } from 'next/navigation'
import { getProjectById } from '@/lib/projects'

export default async function ProjectPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const project = getProjectById(id)

  if (!project) {
    notFound()
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <Link href="/code" className="text-blue-600 hover:underline mb-8 inline-block">
          ← back
        </Link>
        
        <article className="prose prose-lg max-w-none essay-content">
          <div className="mb-8">
            <div className="text-xl text-black font-normal mb-2">{project.title}</div>
            {project.date && (
              <time className="text-gray-600 text-sm">{project.date}</time>
            )}
          </div>
          
          {project.description && (
            <div 
              className="leading-relaxed"
              dangerouslySetInnerHTML={{ __html: project.description }}
            />
          )}
        </article>
      </div>
    </div>
  )
}
