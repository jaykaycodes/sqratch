import { createFileRoute } from '@tanstack/react-router'

import Icons from '#/components/icons'
import { Card } from '#/components/ui/card'

export const Route = createFileRoute('/project/')({
	component: () => <ProjectIndex />,
})

function ProjectIndex() {
	return (
		<div className="flex h-full items-center justify-center p-6">
			<Card className="p-6 text-center">
				<Icons.Database className="mx-auto mb-4 text-muted-foreground" size={48} />
				<h3 className="mb-2 text-lg font-medium">Select a table or query</h3>
				<p className="text-muted-foreground text-sm">
					Choose a table from the sidebar or run a query to see data here
				</p>
			</Card>
		</div>
	)
}
