import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function VersionTab() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Version Components</CardTitle>
        <CardDescription>
          Components and libraries for this instance.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground">
          This tab would show the components/libraries for this instance.
        </p>
      </CardContent>
    </Card>
  );
}
