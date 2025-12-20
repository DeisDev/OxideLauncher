// Notes tab component for adding user notes to an instance
//
// Oxide Launcher — A Rust-based Minecraft launcher
// Copyright (C) 2025 Oxide Launcher contributors
//
// This file is part of Oxide Launcher.
//
// Oxide Launcher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Oxide Launcher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
