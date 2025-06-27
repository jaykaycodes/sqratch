import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/project/t/$id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/project/t/$id"!</div>
}
