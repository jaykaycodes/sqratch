import { createFileRoute, Outlet } from '@tanstack/react-router'

import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '#/components/ui/resizable'
import { useInitProject } from '#/providers/project'
import StatusBar from '#/routes/-project/status-bar'

import ProjectWorkbench from './-project/workbench'

export const Route = createFileRoute('/project')({
	component: () => <ProjectLayout />,
})

function ProjectLayout() {
	useInitProject()

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
