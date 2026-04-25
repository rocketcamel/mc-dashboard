import Header from '@/components/header'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_authenticated')({
  component: Authenticated,
})

function Authenticated() {
  return (
    <>
      <Header />
      <Outlet />
    </>
  )
}
