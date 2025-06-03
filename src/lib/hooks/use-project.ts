import { useLoaderData, useMatchRoute } from '@tanstack/react-router'

export const useIsProjectRoute = () => {
	const match = useMatchRoute()
	return match({ to: '/project', fuzzy: true })
}

export const useProject = () => {
	const isProject = useIsProjectRoute()
	if (!isProject) throw new Error('useProject must be used within a project route')
	return useLoaderData({ from: '/project' }).project
}
