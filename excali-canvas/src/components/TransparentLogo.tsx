import { useEffect, useState } from 'react';
import logo from '@/assets/logo.png';
import { removeBackground, loadImage } from '@/utils/backgroundRemoval';

interface TransparentLogoProps {
  className?: string;
  alt?: string;
}

export const TransparentLogo = ({ className, alt = "ExcalidGPT Logo" }: TransparentLogoProps) => {
  const [processedLogo, setProcessedLogo] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(true);

  useEffect(() => {
    const processLogo = async () => {
      try {
        // Fetch the logo image
        const response = await fetch(logo);
        const blob = await response.blob();
        
        // Load as image element
        const img = await loadImage(blob);
        
        // Remove background
        const processedBlob = await removeBackground(img);
        
        // Create object URL for the processed image
        const url = URL.createObjectURL(processedBlob);
        setProcessedLogo(url);
      } catch (error) {
        console.error('Failed to process logo:', error);
        // Fallback to original logo
        setProcessedLogo(logo);
      } finally {
        setIsProcessing(false);
      }
    };

    processLogo();

    // Cleanup
    return () => {
      if (processedLogo) {
        URL.revokeObjectURL(processedLogo);
      }
    };
  }, []);

  if (isProcessing) {
    return <div className={className} style={{ background: 'transparent' }} />;
  }

  return <img src={processedLogo || logo} alt={alt} className={className} />;
};
