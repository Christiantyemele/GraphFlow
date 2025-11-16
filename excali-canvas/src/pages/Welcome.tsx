import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";
import { ArrowRight, Zap, Download, Palette, Sparkles } from "lucide-react";

export default function Welcome() {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      
      {/* Hero Section */}
      <section className="container py-24 md:py-32 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-accent/5 to-transparent pointer-events-none" />
        <div className="mx-auto max-w-3xl text-center relative">
          <Badge variant="secondary" className="mb-4 shadow-glow">
            <Sparkles className="w-3 h-3 mr-1" />
            AI-Powered Diagramming
          </Badge>
          <h1 className="text-4xl md:text-6xl font-bold mb-6">
            <span className="gradient-text">Turn ideas into diagrams</span> — instantly
          </h1>
          <p className="text-xl text-muted-foreground mb-8">
            Create stunning diagrams for documentation, presentations, and YouTube thumbnails. Perfect for content creators and developers alike.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button size="lg" className="gradient-primary shadow-glow" asChild>
              <Link to="/auth">
                Start Free <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link to="#examples">See Examples</Link>
            </Button>
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="container py-16 md:py-24">
        <div className="mx-auto max-w-5xl">
          <h2 className="text-3xl font-bold text-center mb-12 gradient-text">Everything you need</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            <Card className="border-primary/20 hover:shadow-glow transition-all duration-300 hover:border-primary/40">
              <CardHeader>
                <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center mb-2">
                  <Sparkles className="h-6 w-6 text-primary" />
                </div>
                <CardTitle>Understands your notes</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>
                  AI automatically interprets your text and creates the best diagram type
                </CardDescription>
              </CardContent>
            </Card>

            <Card className="border-accent/20 hover:shadow-glow transition-all duration-300 hover:border-accent/40">
              <CardHeader>
                <div className="h-12 w-12 rounded-lg bg-accent/10 flex items-center justify-center mb-2">
                  <Download className="h-6 w-6 text-accent" />
                </div>
                <CardTitle>Perfect for Content</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>
                  Export high-quality PNG/SVG images ideal for YouTube thumbnails, social media, and presentations
                </CardDescription>
              </CardContent>
            </Card>

            <Card className="border-primary/20 hover:shadow-glow transition-all duration-300 hover:border-primary/40">
              <CardHeader>
                <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center mb-2">
                  <Palette className="h-6 w-6 text-primary" />
                </div>
                <CardTitle>Light & Dark editor</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>
                  Beautiful Excalidraw-style visuals in any theme you prefer
                </CardDescription>
              </CardContent>
            </Card>

            <Card className="border-accent/20 hover:shadow-glow transition-all duration-300 hover:border-accent/40">
              <CardHeader>
                <div className="h-12 w-12 rounded-lg bg-accent/10 flex items-center justify-center mb-2">
                  <Zap className="h-6 w-6 text-accent" />
                </div>
                <CardTitle>Upgrade to Pro</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>
                  Get more credits and faster generation for professional workflows
                </CardDescription>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* How it Works */}
      <section className="container py-16 md:py-24">
        <div className="mx-auto max-w-4xl">
          <h2 className="text-3xl font-bold text-center mb-12">How it works</h2>
          <div className="space-y-8">
            <div className="flex gap-4 items-start">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-accent text-accent-foreground flex items-center justify-center font-bold">
                1
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Describe your idea</h3>
                <p className="text-muted-foreground">
                  Type or paste your notes, workflow, system architecture, or any concept
                </p>
              </div>
            </div>

            <div className="flex gap-4 items-start">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-accent text-accent-foreground flex items-center justify-center font-bold">
                2
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">AI generates diagram</h3>
                <p className="text-muted-foreground">
                  Our AI understands context and creates a clean, professional diagram
                </p>
              </div>
            </div>

            <div className="flex gap-4 items-start">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-accent text-accent-foreground flex items-center justify-center font-bold">
                3
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Export and share</h3>
                <p className="text-muted-foreground">
                  Download as PNG or SVG for YouTube thumbnails, social media posts, presentations, or documentation
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Examples Section */}
      <section id="examples" className="container py-16 md:py-24 bg-secondary/30">
        <div className="mx-auto max-w-4xl">
          <h2 className="text-3xl font-bold text-center mb-12">Example diagrams</h2>
          <div className="grid md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Flow Diagram</CardTitle>
                <CardDescription>User authentication flow with error handling</CardDescription>
              </CardHeader>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>System Architecture</CardTitle>
                <CardDescription>Microservices deployment on AWS</CardDescription>
              </CardHeader>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Sequence Diagram</CardTitle>
                <CardDescription>API request/response lifecycle</CardDescription>
              </CardHeader>
            </Card>
            <Card className="border-accent/40">
              <CardHeader>
                <CardTitle>YouTube Thumbnails</CardTitle>
                <CardDescription>Eye-catching visuals for video content and social media</CardDescription>
              </CardHeader>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Mindmap</CardTitle>
                <CardDescription>Project planning and task breakdown</CardDescription>
              </CardHeader>
            </Card>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="container py-16 md:py-24">
        <div className="mx-auto max-w-2xl text-center relative">
          <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-accent/5 to-transparent blur-3xl pointer-events-none" />
          <div className="relative">
            <h2 className="text-3xl font-bold mb-4 gradient-text">Ready to visualize your ideas?</h2>
            <p className="text-xl text-muted-foreground mb-8">
              Start creating diagrams in seconds. No credit card required.
            </p>
            <Button size="lg" className="gradient-primary shadow-glow" asChild>
              <Link to="/auth">
                Start Free <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
