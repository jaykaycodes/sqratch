import { createFileRoute } from '@tanstack/react-router'

import Workspace from '#/components/workspace'

export const Route = createFileRoute('/project')({
	component: About,
})

function About() {
	return <Workspace />
}
