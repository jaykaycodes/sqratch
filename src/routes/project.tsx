import { use$ } from '@legendapp/state/react'
import { createFileRoute, Outlet } from '@tanstack/react-router'
import { Allotment, LayoutPriority } from 'allotment'

import { taurpc } from '#/lib/utils'
import ProjectProvider, { useProjectStore$ } from '#/providers/project'
import StatusBar from '#/routes/-project/status-bar'

import ProjectDetails from './-project/details'
import ProjectWorkbench from './-project/workbench'

export const Route = createFileRoute('/project')({
	loader: () => taurpc.db.get_connection_string(),
	component: Wrapper,
})

function Wrapper() {
	const connectionString = Route.useLoaderData()
	return (
		<ProjectProvider connectionString={connectionString}>
			<ProjectLayout />
		</ProjectProvider>
	)
}

function ProjectLayout() {
	const project$ = useProjectStore$()
	const showDetailsPanel = use$(project$.ui.detailsPanel.open)

	return (
		<div className="flex size-full flex-col">
			<Allotment
				onVisibleChange={(index, visible) => {
					if (index === 2) project$.ui.detailsPanel.open.set(visible)
				}}
				proportionalLayout={false}
			>
				<Allotment.Pane
					key="workbench-panel"
					minSize={170}
					preferredSize={300}
					priority={LayoutPriority.Low}
				>
					<ProjectWorkbench />
				</Allotment.Pane>

				<Allotment.Pane key="main-panel" minSize={300} priority={LayoutPriority.High}>
					<Outlet />
				</Allotment.Pane>

				<Allotment.Pane
					key="details-panel"
					minSize={170}
					preferredSize={300}
					priority={LayoutPriority.Low}
					snap
					visible={showDetailsPanel}
				>
					<ProjectDetails />
				</Allotment.Pane>
			</Allotment>

			<StatusBar />
		</div>
	)
}
