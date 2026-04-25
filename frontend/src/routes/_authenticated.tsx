import Header from '@/components/header'
import { authQueryOptions } from '@/lib/auth'
import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/_authenticated')({
  component: Authenticated,
  beforeLoad: async ({ context }) => {
    const user = await context.queryClient.ensureQueryData(authQueryOptions)
    if (!user) {
      throw redirect({ to: '/login' })
    }
  }
})

function Authenticated() {
  return (
    <>
      <Header />
      <Outlet />
    </>
  )
}
