import { useState } from "react";
import { Upload, FileText, X, Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useToast } from "@/hooks/use-toast";
import { supabase } from "@/integrations/supabase/client";

interface DocumentUploadProps {
  isPro: boolean;
  onUpgrade: () => void;
  userId: string;
}

export const DocumentUpload = ({ isPro, onUpgrade, userId }: DocumentUploadProps) => {
  const [uploadedFile, setUploadedFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const { toast } = useToast();

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (!isPro) {
      toast({
        title: "Pro Feature",
        description: "Document upload is a Pro feature. Upgrade to unlock!",
        variant: "destructive",
      });
      onUpgrade();
      return;
    }

    // Check file size (10MB limit)
    if (file.size > 10485760) {
      toast({
        title: "File too large",
        description: "Maximum file size is 10MB",
        variant: "destructive",
      });
      return;
    }

    // Check file type
    const allowedTypes = [
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'text/plain',
      'application/vnd.ms-powerpoint',
      'application/vnd.openxmlformats-officedocument.presentationml.presentation'
    ];

    if (!allowedTypes.includes(file.type)) {
      toast({
        title: "Invalid file type",
        description: "Please upload PDF, Word, PowerPoint, or text files",
        variant: "destructive",
      });
      return;
    }

    setUploadedFile(file);
  };

  const handleUpload = async () => {
    if (!uploadedFile || !isPro) return;

    setIsUploading(true);
    try {
      const fileExt = uploadedFile.name.split('.').pop();
      const fileName = `${userId}/${Date.now()}.${fileExt}`;

      const { error: uploadError } = await supabase.storage
        .from('documents')
        .upload(fileName, uploadedFile);

      if (uploadError) throw uploadError;

      toast({
        title: "Document uploaded!",
        description: "Processing your document to generate diagram...",
      });

      // TODO: Process document and generate diagram
      setUploadedFile(null);
    } catch (error) {
      console.error('Upload error:', error);
      toast({
        title: "Upload failed",
        description: "Please try again or contact support",
        variant: "destructive",
      });
    } finally {
      setIsUploading(false);
    }
  };

  const handleRemove = () => {
    setUploadedFile(null);
  };

  return (
    <Card className="p-6 border-accent/20">
      <div className="flex items-start gap-4">
        <div className="h-10 w-10 rounded-lg bg-accent/10 flex items-center justify-center flex-shrink-0">
          {isPro ? (
            <FileText className="h-5 w-5 text-accent" />
          ) : (
            <Lock className="h-5 w-5 text-muted-foreground" />
          )}
        </div>
        
        <div className="flex-1 space-y-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <h3 className="font-semibold">Upload Document</h3>
              {!isPro && (
                <span className="text-xs px-2 py-0.5 rounded-full bg-primary text-primary-foreground">
                  PRO
                </span>
              )}
            </div>
            <p className="text-sm text-muted-foreground">
              Upload PDF, Word, PowerPoint, or text files to generate diagrams automatically
            </p>
          </div>

          {uploadedFile ? (
            <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
              <FileText className="h-5 w-5 text-accent" />
              <span className="text-sm flex-1 truncate">{uploadedFile.name}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleRemove}
                disabled={isUploading}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          ) : (
            <div className="relative">
              <input
                type="file"
                id="document-upload"
                className="hidden"
                accept=".pdf,.doc,.docx,.txt,.ppt,.pptx"
                onChange={handleFileSelect}
              />
              <label htmlFor="document-upload">
                <Button
                  type="button"
                  variant={isPro ? "outline" : "secondary"}
                  className="w-full"
                  asChild
                >
                  <span className="flex items-center justify-center gap-2 cursor-pointer">
                    <Upload className="h-4 w-4" />
                    {isPro ? "Choose Document" : "Upgrade to Upload Documents"}
                  </span>
                </Button>
              </label>
            </div>
          )}

          {uploadedFile && isPro && (
            <Button
              onClick={handleUpload}
              disabled={isUploading}
              className="w-full"
            >
              {isUploading ? "Uploading..." : "Process Document"}
            </Button>
          )}
        </div>
      </div>
    </Card>
  );
};
