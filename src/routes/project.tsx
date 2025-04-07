import React from 'react'

import { use$ } from '@legendapp/state/react'
import { $React } from '@legendapp/state/react-web'
import { createFileRoute } from '@tanstack/react-router'

import Icons from '#/components/icons'
import { Button } from '#/components/ui/button'
import { Card } from '#/components/ui/card'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '#/components/ui/collapsible'
import { ResizableHandle } from '#/components/ui/resizable'
import { ResizablePanel } from '#/components/ui/resizable'
import { ResizablePanelGroup } from '#/components/ui/resizable'
import { ScrollArea } from '#/components/ui/scroll-area'
import { Separator } from '#/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import project$, { useProjectInit } from '#/providers/project'

export const Route = createFileRoute('/project')({
	component: () => <ProjectIndex />,
})

function ProjectIndex() {
	useProjectInit()

	const status = use$(project$.status)
	const connString = use$(project$.connectionString)
	const [showRightPanel, setShowRightPanel] = React.useState(false)

	return (
		<div className="flex h-screen w-full flex-col overflow-hidden">
			{/* Connection status bar */}
			<div className="flex h-9 items-center justify-between border-b bg-muted/40 px-4">
				<div className="flex items-center gap-2">
					<$React.div
						className={`h-2 w-2 rounded-full ${status === 'connected' ? 'bg-green-500' : 'bg-red-500'}`}
					/>
					<span className="text-muted-foreground text-xs">
						{status === 'connected' ? 'Connected' : 'Disconnected'}:{' '}
					</span>
					<span className="text-muted-foreground text-xs">{connString}</span>
				</div>
			</div>

			{/* Main workspace */}
			<ResizablePanelGroup className="flex-1" direction="horizontal">
				{/* Left sidebar */}
				<ResizablePanel className="border-r" defaultSize={20} maxSize={30} minSize={15}>
					<div className="flex h-full flex-col">
						{/* Queries section */}
						<Collapsible className="border-b" defaultOpen>
							<CollapsibleTrigger className="flex h-8 w-full items-center justify-between px-4 py-1 hover:bg-muted/50">
								<div className="flex items-center gap-2">
									<Icons.FileCode size={16} />
									<span className="font-medium text-sm">Queries</span>
								</div>
								<Icons.ChevronDown className="text-muted-foreground" size={16} />
							</CollapsibleTrigger>
							<CollapsibleContent>
								<ScrollArea className="h-[180px] px-4 py-2">
									<div className="space-y-1">
										<Button className="w-full justify-start" size="sm" variant="ghost">
											<Icons.FileText className="mr-2" size={14} />
											<span className="text-xs">Select all users</span>
										</Button>
										<Button className="w-full justify-start" size="sm" variant="ghost">
											<Icons.FileText className="mr-2" size={14} />
											<span className="text-xs">Recent orders</span>
										</Button>
										<Button className="w-full justify-start" size="sm" variant="ghost">
											<Icons.FileText className="mr-2" size={14} />
											<span className="text-xs">Product inventory</span>
										</Button>
									</div>
								</ScrollArea>
							</CollapsibleContent>
						</Collapsible>

						{/* Database objects section */}
						<Collapsible className="flex-1" defaultOpen>
							<CollapsibleTrigger className="flex h-8 w-full items-center justify-between px-4 py-1 hover:bg-muted/50">
								<div className="flex items-center gap-2">
									<Icons.Database size={16} />
									<span className="font-medium text-sm">Database</span>
								</div>
								<Icons.ChevronDown className="text-muted-foreground" size={16} />
							</CollapsibleTrigger>
							<CollapsibleContent className="h-[calc(100%-210px)]">
								<Tabs className="h-full" defaultValue="tables">
									<TabsList className="w-full px-4 py-1">
										<TabsTrigger className="text-xs" value="tables">
											Tables
										</TabsTrigger>
										<TabsTrigger className="text-xs" value="schemas">
											Schemas
										</TabsTrigger>
									</TabsList>
									<TabsContent className="h-[calc(100%-32px)] px-0" value="tables">
										<ScrollArea className="h-full px-2 py-2">
											<div className="space-y-1">
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.Table className="mr-2" size={14} />
													<span className="text-xs">users</span>
												</Button>
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.Table className="mr-2" size={14} />
													<span className="text-xs">products</span>
												</Button>
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.Table className="mr-2" size={14} />
													<span className="text-xs">orders</span>
												</Button>
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.Table className="mr-2" size={14} />
													<span className="text-xs">order_items</span>
												</Button>
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.Table className="mr-2" size={14} />
													<span className="text-xs">categories</span>
												</Button>
											</div>
										</ScrollArea>
									</TabsContent>
									<TabsContent className="h-[calc(100%-32px)] px-0" value="schemas">
										<ScrollArea className="h-full px-2 py-2">
											<div className="space-y-1">
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.FolderOpen className="mr-2" size={14} />
													<span className="text-xs">public</span>
												</Button>
												<Button className="w-full justify-start" size="sm" variant="ghost">
													<Icons.FolderOpen className="mr-2" size={14} />
													<span className="text-xs">auth</span>
												</Button>
											</div>
										</ScrollArea>
									</TabsContent>
								</Tabs>
							</CollapsibleContent>
						</Collapsible>
					</div>
				</ResizablePanel>

				<ResizableHandle withHandle />

				{/* Main content area */}
				<ResizablePanel className="flex-1" defaultSize={60}>
					<div className="flex h-full items-center justify-center p-6">
						<Card className="p-6 text-center">
							<Icons.Database className="mx-auto mb-4 text-muted-foreground" size={48} />
							<h3 className="mb-2 text-lg font-medium">Select a table or query</h3>
							<p className="text-muted-foreground text-sm">
								Choose a table from the sidebar or run a query to see data here
							</p>
						</Card>
					</div>
				</ResizablePanel>

				{/* Right panel - conditionally shown */}
				{showRightPanel && (
					<>
						<ResizableHandle withHandle />
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
			</ResizablePanelGroup>

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
		</div>
	)
}
