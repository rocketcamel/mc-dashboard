import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { login } from '@/lib/auth'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useForm } from "@tanstack/react-form"

export const Route = createFileRoute('/login')({
  component: Login,
})

function Login() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const loginMutation = useMutation({
    mutationFn: (values: { username: string, password: string }) => login(values.username, values.password),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['auth'] })
      navigate({ to: '/' })
    }
  })

  const form = useForm({
    defaultValues: {
      username: '',
      password: '',
    },
    onSubmit: ({ value }) => {
      loginMutation.mutate(value)
    }
  })

  return (
    <div className="flex flex-col max-w-xs mt-6 mx-auto">
      <Card className='pb-0'>
        <CardHeader>
          <CardTitle>Login</CardTitle>
          <CardDescription>Enter username below</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={(e) => {
            e.preventDefault()
            e.stopPropagation()
            form.handleSubmit()
          }} id='login-form'>
            <div className='flex flex-col gap-6'>
              <form.Field name='username'>
                {(field) => (
                  <div className='grid gap-2'>
                    <Label htmlFor='username'>Username</Label>
                    <Input id="email" type="text" placeholder='mrfartshit' value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} required />
                  </div>
                )}
              </form.Field>
              <form.Field name='password'>
                {(field) => (
                  <div className='grid gap-2'>
                    <Label htmlFor='password'>Password</Label>
                    <Input id="password" type="password" placeholder='Password' value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} required />
                  </div>
                )}
              </form.Field>
            </div>
          </form>
        </CardContent>
        <CardFooter className='bg-secondary flex-col p-4 rounded-b-xl border-t'>
          {loginMutation.isPending ? (
            <Button className='w-full' disabled form="login-form">
              <Loader2 className='animate-spin' />
            </Button>
          ) : (
            <Button className='w-full' type="submit" form="login-form">Login</Button>
          )}
        </CardFooter>
      </Card>
    </div>
  )
}
