/**
 * Error Boundary Component
 *
 * React レンダリングエラーをキャッチして白い画面を防止
 */

import { Component } from 'react';
import type { ReactNode, ErrorInfo } from 'react';
import { Button } from './Button';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex h-full items-center justify-center p-8">
          <div className="text-center">
            <p className="mb-2 text-red-600">
              予期しないエラーが発生しました
            </p>
            <p className="mb-4 text-sm text-gray-500">
              {this.state.error?.message ?? '不明なエラー'}
            </p>
            <Button variant="secondary" onClick={this.handleReset}>
              再試行
            </Button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
