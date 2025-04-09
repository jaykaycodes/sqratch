import React from 'react'

import { use$ } from '@legendapp/state/react'
import { createFileRoute } from '@tanstack/react-router'

import Icons from '#/components/icons'
import { Button } from '#/components/ui/button'
import { Card } from '#/components/ui/card'
import { ResizableHandle, ResizablePanel } from '#/components/ui/resizable'
import { ScrollArea } from '#/components/ui/scroll-area'
import { Separator } from '#/components/ui/separator'
import project$ from '#/providers/project'

export const Route = createFileRoute('/project/')({
	component: () => <ProjectIndex />,
})

function ProjectIndex() {
	const status = use$(project$.status)
	const connString = use$(project$.connectionString)
	const [showRightPanel, setShowRightPanel] = React.useState(false)

	return (
		<>
			<ResizablePanel className="relative flex-1" defaultSize={60}>
				<div className="flex h-full items-center justify-center p-6">
					<Card className="p-6 text-center">
						<Icons.Database className="mx-auto mb-4 text-muted-foreground" size={48} />
						<h3 className="mb-2 text-lg font-medium">Select a table or query</h3>
						<p className="text-muted-foreground text-sm">
							Choose a table from the sidebar or run a query to see data here
						</p>
					</Card>
				</div>

				{/* Show right panel button - only visible when panel is hidden */}
				{!showRightPanel && (
					<Button
						className="absolute bottom-4 right-4 z-10"
						onClick={() => setShowRightPanel(true)}
						size="sm"
						variant="outline"
					>
						<Icons.PanelRight className="mr-2" size={16} />
						Show Details
					</Button>
				)}
			</ResizablePanel>

			{/* Right panel - conditionally shown */}
			{showRightPanel && (
				<>
					<ResizableHandle />
					<ResizablePanel defaultSize={20} maxSize={40} minSize={15}>
						<div className="flex h-full flex-col">
							<div className="flex h-9 items-center justify-between border-b px-4">
								<h3 className="text-sm font-medium">Row Details</h3>
								<Button
									className="h-6 w-6"
									onClick={() => setShowRightPanel(false)}
									size="icon"
									variant="ghost"
								>
									<Icons.X size={16} />
								</Button>
							</div>
							<ScrollArea className="flex-1 p-4">
								<div className="space-y-4">
									<div>
										<p className="text-xs text-muted-foreground">id</p>
										<p className="text-sm">1</p>
									</div>
									<Separator />
									<div>
										<p className="text-xs text-muted-foreground">name</p>
										<p className="text-sm">Example Name</p>
									</div>
									<Separator />
									<div>
										<p className="text-xs text-muted-foreground">created_at</p>
										<p className="text-sm">2025-04-04T16:00:00Z</p>
									</div>
								</div>
							</ScrollArea>
						</div>
					</ResizablePanel>
				</>
			)}
		</>
	)
}
