import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";

interface NotesTabProps {
  notes: string;
  setNotes: (notes: string) => void;
}

export function NotesTab({ notes, setNotes }: NotesTabProps) {
  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Notes</CardTitle>
        <CardDescription>Add notes about this instance.</CardDescription>
      </CardHeader>
      <CardContent>
        <Textarea
          placeholder="Add notes about this instance..."
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          className="min-h-[300px]"
        />
      </CardContent>
    </Card>
  );
}
