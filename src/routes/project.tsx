import { createFileRoute, Outlet } from '@tanstack/react-router'

import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '#/components/ui/resizable'
import { taurpc } from '#/lib/utils'
import { DbProvider } from '#/providers/db'
import StatusBar from '#/routes/-project/status-bar'

import ProjectWorkbench from './-project/workbench'

export const Route = createFileRoute('/project')({
	loader: () => taurpc.db.get_connection_string(),
	component: () => {
		const connectionString = Route.useLoaderData()

		return (
			<DbProvider connectionString={connectionString}>
				<ProjectLayout />
			</DbProvider>
		)
	},
})

function ProjectLayout() {
	return (
		<div className="flex h-full w-full flex-col">
			<ResizablePanelGroup className="h-full flex-1" direction="horizontal">
				<ResizablePanel defaultSize={20} maxSize={30} minSize={15}>
					<ProjectWorkbench />
				</ResizablePanel>

				<ResizableHandle />

				<Outlet />
			</ResizablePanelGroup>

			<StatusBar />
		</div>
	)
}
