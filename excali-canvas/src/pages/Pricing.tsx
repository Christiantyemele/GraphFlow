import { Link } from "react-router-dom";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";
import { Check } from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { supabase } from "@/integrations/supabase/client";
import { useToast } from "@/hooks/use-toast";

export default function Pricing() {
  const [isCheckingOut, setIsCheckingOut] = useState(false);
  const { user, subscriptionStatus } = useAuth();
  const { toast } = useToast();

  const handleUpgrade = async () => {
    if (!user) {
      window.location.href = "/auth";
      return;
    }

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

  const isPro = subscriptionStatus?.subscribed || false;

  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      
      <section className="container py-24 md:py-32 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-accent/5 to-transparent pointer-events-none" />
        <div className="mx-auto max-w-5xl relative">
          <div className="text-center mb-12">
            <h1 className="text-4xl md:text-5xl font-bold mb-4">
              <span className="gradient-text">Simple, transparent</span> pricing
            </h1>
            <p className="text-xl text-muted-foreground">
              Start free, upgrade when you need more
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
            {/* Free Plan */}
            <Card className="border-2 hover:shadow-glow transition-all duration-300">
              <CardHeader>
                <CardTitle className="text-2xl">Free</CardTitle>
                <CardDescription>Perfect for trying out ExcalidGPT</CardDescription>
                <div className="mt-4">
                  <span className="text-4xl font-bold">$0</span>
                  <span className="text-muted-foreground">/month</span>
                </div>
              </CardHeader>
              <CardContent>
                <ul className="space-y-3">
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span><span className="font-semibold">10 credits</span> per month</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>All diagram types (flow, system, sequence, mindmap)</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>PNG & SVG exports</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>Standard generation speed</span>
                  </li>
                </ul>
              </CardContent>
              <CardFooter>
                <Button variant="outline" className="w-full" asChild>
                  <Link to="/app/editor">Start Free</Link>
                </Button>
              </CardFooter>
            </Card>

            {/* Pro Plan */}
            <Card className="border-2 border-primary relative shadow-glow">
              {isPro ? (
                <Badge className="absolute -top-3 left-1/2 -translate-x-1/2 bg-green-500">
                  Your Plan
                </Badge>
              ) : (
                <Badge className="absolute -top-3 left-1/2 -translate-x-1/2 gradient-primary shadow-glow">
                  Most Popular
                </Badge>
              )}
              <CardHeader>
                <CardTitle className="text-2xl gradient-text">Pro</CardTitle>
                <CardDescription>For power users and teams</CardDescription>
                <div className="mt-4">
                  <span className="text-4xl font-bold gradient-text">$4.99</span>
                  <span className="text-muted-foreground">/month</span>
                </div>
              </CardHeader>
              <CardContent>
                <ul className="space-y-3">
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span className="font-semibold">1,000 credits per month</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>All diagram types (flow, system, sequence, mindmap)</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>PNG & SVG exports</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span className="font-semibold">Priority generation (3x faster)</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <Check className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                    <span>Priority support</span>
                  </li>
                </ul>
              </CardContent>
              <CardFooter>
                {isPro ? (
                  <Button className="w-full" variant="outline" asChild>
                    <Link to="/app/editor">Go to Editor</Link>
                  </Button>
                ) : (
                  <Button
                    className="w-full gradient-primary shadow-glow"
                    onClick={handleUpgrade}
                    disabled={isCheckingOut}
                  >
                    {isCheckingOut ? "Loading..." : "Upgrade to Pro"}
                  </Button>
                )}
              </CardFooter>
            </Card>
          </div>

          {/* FAQ */}
          <div className="mt-24 max-w-3xl mx-auto">
            <h2 className="text-3xl font-bold text-center mb-12">Frequently asked questions</h2>
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold mb-2">What are credits?</h3>
                <p className="text-muted-foreground">
                  Credits are used each time you generate a diagram. Simple diagrams use 1 credit, more complex ones may use 2-3 credits.
                </p>
              </div>
              <div>
                <h3 className="text-lg font-semibold mb-2">Can I cancel anytime?</h3>
                <p className="text-muted-foreground">
                  Yes! You can cancel your Pro subscription at any time. You'll keep access until the end of your billing period.
                </p>
              </div>
              <div>
                <h3 className="text-lg font-semibold mb-2">Do unused credits roll over?</h3>
                <p className="text-muted-foreground">
                  No, credits reset at the start of each billing period. However, Pro users get 10x more credits than they typically need.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
