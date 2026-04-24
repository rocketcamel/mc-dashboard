import Backup from '@/components/backup'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { createFileRoute, redirect } from '@tanstack/react-router'
import { authQueryOptions } from '@/lib/auth'

export const Route = createFileRoute('/')({
  component: Index,
  beforeLoad: async ({ context }) => {
    const user = await context.queryClient.ensureQueryData(authQueryOptions)
    if (!user) {
      throw redirect({ to: '/login' })
    }
  }
})

function Index() {
  return (
    <div className='flex flex-col items-center max-w-334 mx-auto mt-6'>
      <h1 className='text-2xl font-semibold border-b-2 pb-1 px-6'>Management</h1>
      <div className='grid grid-cols-1 lg:grid-cols-3 gap-6 p-6'>
        <Card className='lg:col-span-3'>
          <CardHeader>
            <CardTitle>Minecraft</CardTitle>
            <CardDescription>Minecraft Server Console</CardDescription>
          </CardHeader>
          <CardContent className='grid grid-cols-2 gap-4'>
            <Button>Sync Creative World</Button>
            <Backup />
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
