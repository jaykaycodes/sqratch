import { ScrollArea } from '#/components/ui/scroll-area'
import { Separator } from '#/components/ui/separator'

export default function ProjectDetails() {
	return (
		<div className="flex w-full h-full flex-col">
			<div className="flex h-9 items-center justify-between border-b px-4">
				<h3 className="text-sm font-medium">Row Details</h3>
			</div>
			<ScrollArea className="flex-1 p-4">
				<div className="space-y-4">
					<div>
						<p className="text-muted-foreground">id</p>
						<p className="text-sm">1</p>
					</div>
					<Separator />
					<div>
						<p className="text-muted-foreground">name</p>
						<p className="text-sm">Example Name</p>
					</div>
					<Separator />
					<div>
						<p className="text-muted-foreground">created_at</p>
						<p className="text-sm">2025-04-04T16:00:00Z</p>
					</div>
				</div>
			</ScrollArea>
		</div>
	)
}
