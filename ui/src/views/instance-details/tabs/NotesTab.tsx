import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";

interface NotesTabProps {
  notes: string;
  setNotes: (notes: string) => void;
}

export function NotesTab({ notes, setNotes }: NotesTabProps) {
  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader className="flex-shrink-0">
        <CardTitle>Notes</CardTitle>
        <CardDescription>Add notes about this instance.</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-hidden">
        <Textarea
          placeholder="Add notes about this instance..."
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          className="h-full min-h-[200px] max-h-full resize-none"
        />
      </CardContent>
    </Card>
  );
}
