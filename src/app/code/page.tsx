import Link from 'next/link'
import { getAllProjects } from '@/lib/projects'

export default function Code() {
  const projects = getAllProjects()

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl font-normal mb-8">Vivien Henz</h1>
        
        <p className="text-gray-700 mb-8 max-w-xl">
          Here are my coding projects. If you&apos;d rather read essays, click{' '}
          <Link href="/" className="text-blue-600 underline hover:text-blue-800">
            here
          </Link>
          .
        </p>
        
        
        <div className="space-y-3">
          {projects.map((project) => (
            <div key={project.id}>
              <Link 
                href={`/code/${project.id}`}
                className="text-blue-600 underline hover:text-blue-800"
              >
                {project.title}
              </Link>
            </div>
          ))}
        </div>
        
        {projects.length === 0 && (
          <p className="text-gray-500 text-center py-8">
            No projects yet. Add some to the projects data.
          </p>
        )}
      </div>
    </div>
  )
}
