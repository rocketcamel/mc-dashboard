import Backup from '@/components/backup'
import StatusBadge from '@/components/status-badge'
import Sync from '@/components/sync'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { statusQueryOptions } from '@/lib/backup'
import { get_world_statuses } from '@/lib/world'
import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'

export const Route = createFileRoute('/_authenticated/')({
  component: Index,
})

function Index() {
  const { data: isOperationRunning } = useQuery(statusQueryOptions)
  const { data: statuses } = useQuery({
    queryKey: ["status", "world"],
    queryFn: get_world_statuses,
    staleTime: 20 * 1000,
    refetchInterval: 20 * 1000
  })

  return (
    <div className='flex flex-col items-center max-w-334 mx-auto mt-6'>
      <h1 className='text-2xl font-semibold border-b-2 pb-1 px-6'>Management</h1>
      <div className='grid grid-cols-1 lg:grid-cols-3 gap-6 p-6'>
        <Card className='lg:col-span-3'>
          <CardHeader className='flex flex-row justify-between'>
            <div>
              <CardTitle>Minecraft</CardTitle>
              <CardDescription>Minecraft Server Console</CardDescription>
            </div>
            {isOperationRunning && (
              <div className='flex items-center gap-2 text-sm text-muted-foreground'>
                <Loader2 className='h-4 w-4 animate-spin' />
                Running...
              </div>
            )}
          </CardHeader>
          <CardContent className='grid grid-cols-2 gap-4'>
            <div className='flex flex-col gap-2'>
              <div className='flex items-center gap-2'>
                <span className='text-sm font-medium'>Creative</span>
                <StatusBadge status={statuses?.creative} />
              </div>
              <Sync disabled={isOperationRunning} />
            </div>
            <div className='flex flex-col gap-2'>
              <div className='flex items-center gap-2'>
                <span className='text-sm font-medium'>Main</span>
                <StatusBadge status={statuses?.main} />
              </div>
              <Backup disabled={isOperationRunning} />
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
