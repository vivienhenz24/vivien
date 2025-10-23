import Link from 'next/link'
import { getAllContent } from '@/lib/content'

export default function Home() {
  const content = getAllContent()

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl font-normal mb-8">Vivien Henz</h1>
        
        <p className="text-gray-700 mb-8 max-w-xl">
          Hi! I like to work hard on things I find interesting. So here are my essays and coding projects about them, which I hope you find interesting too.
          <br /><br />
          Let&apos;s connect on <a href="https://github.com/vivienhenz24" className="text-blue-600 underline hover:text-blue-800">GitHub</a>, <a href="https://linkedin.com/in/vivienhenz" className="text-blue-600 underline hover:text-blue-800">linkedIn</a>, or vhenz@college.harvard.edu!
        </p>
        
        <div className="space-y-3">
          {content.map((item) => (
            <div key={item.id}>
              <Link href={`/${item.id}`} className="text-blue-600 underline hover:text-blue-800">
                {item.title}
              </Link>
            </div>
          ))}
        </div>
        
        {content.length === 0 && (
          <p className="text-gray-500 text-center py-8">
            No content yet. Add some essays or projects.
          </p>
        )}
      </div>
    </div>
  )
}
