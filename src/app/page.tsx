import Link from 'next/link'
import { getAllEssays } from '@/lib/essays'

export default function Home() {
  const essays = getAllEssays()

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl font-normal mb-8">Vivien Henz</h1>
        
        <p className="text-gray-700 mb-8 max-w-xl">
          Here are my essays. If you'd rather read code, click{' '}
          <Link href="/code" className="text-blue-600 underline hover:text-blue-800">
            here
          </Link>
          .
        </p>
        
        <div className="space-y-3">
          {essays.map((essay) => (
            <div key={essay.id}>
              <Link href={`/${essay.id}`} className="text-blue-600 underline hover:text-blue-800">
                {essay.title}
              </Link>
            </div>
          ))}
        </div>
        
        {essays.length === 0 && (
          <p className="text-gray-500 text-center py-8">
            No essays yet. Add some markdown files to the src/content/essays directory.
          </p>
        )}
      </div>
    </div>
  )
}
