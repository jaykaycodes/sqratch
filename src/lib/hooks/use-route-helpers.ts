import { useMatchRoute } from '@tanstack/react-router'

export const useIsProjectRoute = () => {
	const match = useMatchRoute()
	return match({ to: '/project', fuzzy: true })
}
