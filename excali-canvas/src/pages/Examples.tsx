import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";
import { ArrowRight, Workflow, Network, GitBranch, Brain, Database, Users, Calendar, ShoppingCart } from "lucide-react";
import { useNavigate } from "react-router-dom";

const Examples = () => {
  const navigate = useNavigate();

  const examples = [
    {
      title: "User Authentication Flow",
      description: "Complete authentication system with login, signup, and password reset flows",
      category: "Flow Diagram",
      icon: Users,
      prompt: "Create a user authentication flow diagram showing login, signup, password reset, and email verification processes",
      complexity: "Medium",
    },
    {
      title: "E-commerce System Architecture",
      description: "Microservices architecture for a scalable e-commerce platform",
      category: "System Architecture",
      icon: ShoppingCart,
      prompt: "Design a microservices architecture for an e-commerce platform with payment processing, inventory management, and order tracking",
      complexity: "Advanced",
    },
    {
      title: "API Request Sequence",
      description: "RESTful API interaction between client, server, and database",
      category: "Sequence Diagram",
      icon: Network,
      prompt: "Show the sequence of API calls for a user creating a new post, including authentication, validation, and database storage",
      complexity: "Simple",
    },
    {
      title: "CI/CD Pipeline",
      description: "Automated deployment pipeline from code commit to production",
      category: "Flow Diagram",
      icon: GitBranch,
      prompt: "Illustrate a CI/CD pipeline with testing, building, staging, and deployment stages for a web application",
      complexity: "Medium",
    },
    {
      title: "Database Schema Design",
      description: "Relational database structure for a social media application",
      category: "Database Diagram",
      icon: Database,
      prompt: "Design a database schema for a social media app with users, posts, comments, likes, and followers",
      complexity: "Medium",
    },
    {
      title: "Product Development Mindmap",
      description: "Feature planning and roadmap organization",
      category: "Mindmap",
      icon: Brain,
      prompt: "Create a mindmap for product development including features, user stories, technical requirements, and milestones",
      complexity: "Simple",
    },
    {
      title: "Event-Driven Architecture",
      description: "Message queue system with multiple microservices",
      category: "System Architecture",
      icon: Workflow,
      prompt: "Design an event-driven architecture using message queues for real-time notifications and data processing",
      complexity: "Advanced",
    },
    {
      title: "Sprint Planning Workflow",
      description: "Agile sprint cycle from planning to retrospective",
      category: "Flow Diagram",
      icon: Calendar,
      prompt: "Map out a complete sprint cycle including planning, daily standups, development, review, and retrospective",
      complexity: "Simple",
    },
  ];

  const getComplexityColor = (complexity: string) => {
    switch (complexity) {
      case "Simple":
        return "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20";
      case "Medium":
        return "bg-yellow-500/10 text-yellow-700 dark:text-yellow-400 border-yellow-500/20";
      case "Advanced":
        return "bg-orange-500/10 text-orange-700 dark:text-orange-400 border-orange-500/20";
      default:
        return "bg-muted text-muted-foreground";
    }
  };

  return (
    <div className="min-h-screen bg-background flex flex-col">
      <Navbar />
      
      <main className="flex-1">
        {/* Hero Section */}
        <section className="py-20 px-4 bg-gradient-to-b from-primary/5 to-background">
          <div className="container mx-auto max-w-6xl text-center">
            <Badge className="mb-4" variant="secondary">
              Example Generations
            </Badge>
            <h1 className="text-4xl md:text-6xl font-bold mb-6 bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
              Get Inspired by Examples
            </h1>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">
              Explore what you can create with AI-powered diagramming. From simple flows to complex architectures.
            </p>
            <Button size="lg" onClick={() => navigate("/app/editor")}>
              Start Creating
              <ArrowRight className="ml-2 h-4 w-4" />
            </Button>
          </div>
        </section>

        {/* Examples Grid */}
        <section className="py-16 px-4">
          <div className="container mx-auto max-w-7xl">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {examples.map((example, index) => {
                const Icon = example.icon;
                return (
                  <Card 
                    key={index} 
                    className="group hover:shadow-lg transition-all duration-300 hover:scale-105 cursor-pointer border-border/50"
                    onClick={() => navigate("/app/editor")}
                  >
                    <CardHeader>
                      <div className="flex items-start justify-between mb-2">
                        <div className="p-2 rounded-lg bg-primary/10 text-primary">
                          <Icon className="h-6 w-6" />
                        </div>
                        <Badge 
                          variant="outline" 
                          className={getComplexityColor(example.complexity)}
                        >
                          {example.complexity}
                        </Badge>
                      </div>
                      <CardTitle className="text-xl group-hover:text-primary transition-colors">
                        {example.title}
                      </CardTitle>
                      <CardDescription className="text-sm">
                        {example.description}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        <div>
                          <Badge variant="secondary" className="text-xs">
                            {example.category}
                          </Badge>
                        </div>
                        <div className="p-3 bg-muted rounded-md border border-border/50">
                          <p className="text-sm text-muted-foreground italic">
                            "{example.prompt}"
                          </p>
                        </div>
                        <Button 
                          variant="ghost" 
                          className="w-full group-hover:bg-primary group-hover:text-primary-foreground transition-colors"
                          onClick={() => navigate("/app/editor")}
                        >
                          Try This Example
                          <ArrowRight className="ml-2 h-4 w-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 px-4">
          <div className="container mx-auto max-w-4xl text-center">
            <div className="bg-gradient-to-r from-primary/10 via-primary/5 to-primary/10 rounded-2xl p-12 border border-primary/20">
              <h2 className="text-3xl md:text-4xl font-bold mb-4">
                Ready to Create Your Own?
              </h2>
              <p className="text-lg text-muted-foreground mb-8">
                Start generating professional diagrams in seconds with AI assistance.
              </p>
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" onClick={() => navigate("/app/editor")}>
                  Start Free
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
                <Button size="lg" variant="outline" onClick={() => navigate("/pricing")}>
                  View Pricing
                </Button>
              </div>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
};

export default Examples;
