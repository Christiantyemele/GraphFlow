import { useState, useEffect, useRef } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Loader2, Download, Sparkles, RefreshCw, Paperclip } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/contexts/AuthContext";
import { supabase } from "@/integrations/supabase/client";
import { api } from "@/integrations/api/client";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const examplePrompts = [
  { label: "Flow", text: "User login flow with email verification and password reset", type: "flowchart" },
  { label: "System", text: "E-commerce platform with microservices architecture on AWS", type: "system" },
  { label: "Sequence", text: "Payment processing API flow with Stripe integration", type: "sequence" },
  { label: "Mindmap", text: "Product launch planning including marketing, development, and operations", type: "mindmap" },
  { label: "Picture", text: "A futuristic dashboard with holographic displays and neon lights", type: "picture" },
];

interface Message {
  role: "user" | "assistant";
  content: string;
  imageUrl?: string;
  mermaidCode?: string;
  type?: "text" | "image" | "diagram";
}

export default function Editor() {
  const [content, setContent] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [currentType, setCurrentType] = useState<string>("flowchart");
  const [isCheckingOut, setIsCheckingOut] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { toast } = useToast();
  const { user, signOut, subscriptionStatus, checkSubscription, isLoading } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isGenerating]);

  // Redirect to auth if not logged in
  useEffect(() => {
    if (!isLoading && !user) {
      navigate("/auth");
    }
  }, [user, isLoading, navigate]);

  // Check for success parameter and show toast
  useEffect(() => {
    if (searchParams.get("success") === "true") {
      toast({
        title: "Welcome to Pro!",
        description: "Your subscription is now active. Refreshing status...",
      });
      // Remove the success parameter from URL
      navigate("/app/editor", { replace: true });
      // Trigger a manual check after a short delay
      setTimeout(() => {
        checkSubscription();
      }, 2000);
    }
  }, [searchParams, toast, navigate, checkSubscription]);

  if (isLoading || !user) {
    return null;
  }

  const isPro = subscriptionStatus?.subscribed || false;
  const credits = isPro ? 1000 : 10;

  const handleGenerate = async () => {
    if (!content.trim()) {
      toast({
        title: "Input required",
        description: "Please enter a description to generate",
        variant: "destructive",
      });
      return;
    }

    const userMessage: Message = {
      role: "user",
      content: content,
      type: "text"
    };

    setMessages(prev => [...prev, userMessage]);
    setContent("");
    setIsGenerating(true);

    try {
      const data = await api.generateGraph({
        content: content.trim(),
        tier: isPro ? "pro" : "free",
        allow_images: true,
      });

      // Store the GraphFlow response (graph_data + optional scene)
      const assistantMessage: Message = {
        role: "assistant",
        content: JSON.stringify(data.graph_data, null, 2),
        type: "diagram",
        mermaidCode: data.scene ? JSON.stringify(data.scene, null, 2) : undefined
      };

      setMessages(prev => [...prev, assistantMessage]);
      
      toast({
        title: "Graph generated!",
        description: "You can ask for modifications or generate something new",
      });
      
      // Focus input for next message
      inputRef.current?.focus();
    } catch (error: any) {
      console.error("Generation error:", error);
      toast({
        title: "Generation failed",
        description: error?.message || "Please try again or contact support",
        variant: "destructive",
      });
    } finally {
      setIsGenerating(false);
    }
  };

  const handleExampleClick = (text: string, type: string) => {
    setContent(text);
    setCurrentType(type);
    setTimeout(() => {
      inputRef.current?.focus();
    }, 100);
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (!isPro) {
      toast({
        title: "Pro Feature",
        description: "Document upload is a Pro feature. Upgrade to unlock!",
        variant: "destructive",
      });
      handleUpgrade();
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

    // Upload and process the file
    uploadDocument(file);
  };

  const uploadDocument = async (file: File) => {
    setIsGenerating(true);
    try {
      const fileExt = file.name.split('.').pop();
      const fileName = `${user.id}/${Date.now()}.${fileExt}`;

      const { error: uploadError } = await supabase.storage
        .from('documents')
        .upload(fileName, file);

      if (uploadError) throw uploadError;

      toast({
        title: "Document uploaded!",
        description: "Processing your document...",
      });

      // TODO: Process document and generate diagram
      setIsGenerating(false);
    } catch (error) {
      console.error('Upload error:', error);
      toast({
        title: "Upload failed",
        description: "Please try again or contact support",
        variant: "destructive",
      });
      setIsGenerating(false);
    }
  };

  const handleUpgrade = async () => {
    setIsCheckingOut(true);
    try {
      const { data, error } = await supabase.functions.invoke('create-checkout', {
        headers: {
          Authorization: `Bearer ${(await supabase.auth.getSession()).data.session?.access_token}`,
        },
      });

      if (error) throw error;
      
      if (data?.url) {
        window.open(data.url, '_blank');
      }
    } catch (error) {
      toast({
        title: "Checkout failed",
        description: "Please try again or contact support",
        variant: "destructive",
      });
    } finally {
      setIsCheckingOut(false);
    }
  };

  const handleManageSubscription = async () => {
    try {
      const { data, error } = await supabase.functions.invoke('customer-portal', {
        headers: {
          Authorization: `Bearer ${(await supabase.auth.getSession()).data.session?.access_token}`,
        },
      });

      if (error) throw error;
      
      if (data?.url) {
        window.open(data.url, '_blank');
      }
    } catch (error) {
      toast({
        title: "Failed to open portal",
        description: "Please try again or contact support",
        variant: "destructive",
      });
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-background/80 backdrop-blur-sm sticky top-0 z-50">
        <div className="container flex h-14 items-center justify-between">
          <div className="flex items-center gap-3">
            <h1 className="text-lg font-bold gradient-text">ExcalidGPT</h1>
            <Badge variant={isPro ? "default" : "secondary"} className={isPro ? "gradient-primary shadow-glow text-xs" : "text-xs"}>
              {isPro ? "Pro" : "Free"} • {credits}
            </Badge>
          </div>
          <div className="flex items-center gap-2">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={checkSubscription}
                  >
                    <RefreshCw className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Refresh status</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            {!isPro && (
              <Button
                variant="outline"
                size="sm"
                className="border-primary text-primary hover:bg-primary hover:text-primary-foreground"
                onClick={handleUpgrade}
                disabled={isCheckingOut}
              >
                {isCheckingOut ? "Loading..." : "Upgrade"}
              </Button>
            )}
            <Button variant="ghost" size="sm" onClick={signOut}>
              Sign Out
            </Button>
          </div>
        </div>
      </header>

      {/* Split Layout */}
      <div className="container py-4 h-[calc(100vh-7rem)]">
        <div className="grid grid-cols-3 gap-4 h-full">
        {/* Left Side - Chat Interface */}
        <Card className="h-full flex flex-col shadow-sm col-span-1">
            {/* Chat Header */}
            <div className="p-3 border-b border-border bg-muted/30">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-base font-semibold">Conversation</h2>
                  <p className="text-xs text-muted-foreground">
                    Chat with AI to create and refine
                  </p>
                </div>
                {messages.length > 0 && (
                  <Button 
                    variant="ghost" 
                    size="sm"
                    onClick={() => {
                      setMessages([]);
                      setCurrentType("flowchart");
                    }}
                  >
                    <RefreshCw className="mr-2 h-3 w-3" />
                    New
                  </Button>
                )}
              </div>
            </div>

            {/* Example Prompts Bar */}
            {messages.length === 0 && (
              <div className="p-3 border-b border-border bg-gradient-to-r from-muted/50 to-muted/30">
                <p className="text-[10px] font-semibold text-muted-foreground mb-2 uppercase tracking-wider">Examples</p>
                <div className="flex flex-wrap gap-1.5">
                  {examplePrompts.map((example) => (
                    <Button
                      key={example.label}
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs"
                      onClick={() => handleExampleClick(example.text, example.type)}
                    >
                      {example.label}
                    </Button>
                  ))}
                </div>
              </div>
            )}

            {/* Messages Area */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
              {messages.length === 0 ? (
                <div className="h-full flex items-center justify-center text-center">
                  <div>
                    <Sparkles className="h-12 w-12 text-primary mx-auto mb-3" />
                    <h3 className="text-lg font-semibold mb-2">Start Creating</h3>
                    <p className="text-sm text-muted-foreground max-w-sm">
                      Pick an example or describe what you want to create
                    </p>
                  </div>
                </div>
              ) : (
                messages.map((msg, idx) => (
                  <div
                    key={idx}
                    className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
                  >
                    <div
                      className={`max-w-[85%] rounded-lg p-3 ${
                        msg.role === "user"
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted"
                      }`}
                    >
                      <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                    </div>
                  </div>
                ))
              )}
              
              {isGenerating && (
                <div className="flex justify-start">
                  <div className="max-w-[85%] rounded-lg p-3 bg-muted">
                    <div className="flex items-center gap-2">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span className="text-sm">
                        {currentType === "picture" ? "Generating image..." : "Generating diagram..."}
                      </span>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>

            {/* Input Area */}
            <div className="p-3 border-t border-border bg-muted/20">
              <div className="flex gap-2 items-end">
                <div className="flex-1 relative">
                  <Textarea
                    ref={inputRef}
                    placeholder="Type your message..."
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleGenerate();
                      }
                    }}
                    className="resize-none min-h-[44px] max-h-[120px] pr-10"
                    rows={1}
                  />
                  <input
                    ref={fileInputRef}
                    type="file"
                    className="hidden"
                    accept=".pdf,.doc,.docx,.txt,.ppt,.pptx"
                    onChange={handleFileUpload}
                  />
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="absolute right-1 bottom-1 h-8 w-8"
                          onClick={() => fileInputRef.current?.click()}
                          disabled={isGenerating}
                        >
                          <Paperclip className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent side="top">
                        <p className="text-xs">
                          {isPro ? "Upload document (PDF, Word, etc.)" : "Upload document (Pro only)"}
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <Button
                  onClick={handleGenerate}
                  disabled={isGenerating || !content.trim()}
                  className="gradient-primary shadow-glow h-[44px] px-4"
                >
                  {isGenerating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Sparkles className="h-4 w-4" />
                  )}
                </Button>
              </div>
              <p className="text-[10px] text-muted-foreground mt-1.5 px-1">
                Enter to send • Shift+Enter for new line
              </p>
            </div>
          </Card>

        {/* Right Side - Preview */}
        <Card className="h-full flex flex-col shadow-sm col-span-2">
            <div className="p-3 border-b border-border bg-muted/30">
              <h2 className="text-base font-semibold">Preview</h2>
              <p className="text-xs text-muted-foreground">
                Generated content appears here
              </p>
            </div>
            
            <div className="flex-1 overflow-y-auto p-6">
              {messages.length === 0 || !messages.some(m => m.role === "assistant") ? (
                <div className="h-full flex items-center justify-center text-center">
                  <div>
                    <div className="w-24 h-24 bg-muted rounded-lg mx-auto mb-4 flex items-center justify-center">
                      <Sparkles className="h-12 w-12 text-muted-foreground" />
                    </div>
                    <p className="text-muted-foreground">
                      Generated content will appear here
                    </p>
                  </div>
                </div>
              ) : (
                <div className="space-y-6">
                  {messages
                    .filter(msg => msg.role === "assistant")
                    .map((msg, idx) => (
                      <div key={idx} className="space-y-3">
                        {msg.type === "image" && msg.imageUrl ? (
                          <div className="space-y-3">
                            <img
                              src={msg.imageUrl}
                              alt="Generated image"
                              className="w-full rounded-lg border border-border"
                            />
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                const link = document.createElement('a');
                                link.href = msg.imageUrl!;
                                link.download = 'generated-image.png';
                                link.click();
                              }}
                            >
                              <Download className="mr-2 h-4 w-4" />
                              Download Image
                            </Button>
                          </div>
                        ) : msg.type === "diagram" && msg.mermaidCode ? (
                          <div className="space-y-3">
                            <div className="text-xs font-semibold text-muted-foreground mb-2">
                              MERMAID CODE
                            </div>
                            <pre className="bg-muted p-4 rounded-lg text-xs overflow-x-auto border border-border">
                              <code>{msg.mermaidCode}</code>
                            </pre>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                navigator.clipboard.writeText(msg.mermaidCode!);
                                toast({
                                  title: "Copied!",
                                  description: "Mermaid code copied to clipboard",
                                });
                              }}
                            >
                              <Download className="mr-2 h-4 w-4" />
                              Copy Code
                            </Button>
                          </div>
                        ) : null}
                      </div>
                    ))}
                </div>
              )}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
