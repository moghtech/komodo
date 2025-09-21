import React, { useState, useEffect, useRef, useCallback } from 'react';

interface ScrollFollowProps {
  startFollowing?: boolean;
  render: (props: {
    follow: boolean;
    onScroll: (event: any) => void;
  }) => React.ReactNode;
}

const ScrollFollow: React.FC<ScrollFollowProps> = ({
  startFollowing = true,
  render
}) => {
  const [follow, setFollow] = useState(startFollowing);
  const scrollPositionRef = useRef({ scrollTop: 0, scrollHeight: 0, clientHeight: 0 });

  const handleScroll = useCallback((event: any) => {
    const target = event.currentTarget || event.target;

    if (!target) return;

    const { scrollTop, scrollHeight, clientHeight } = target;
    scrollPositionRef.current = { scrollTop, scrollHeight, clientHeight };

    // Check if user is at the bottom (with a small threshold)
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;

    // If user scrolls up from bottom, stop following
    if (follow && !isAtBottom) {
      setFollow(false);
    }
    // If user scrolls to bottom, start following again
    else if (!follow && isAtBottom) {
      setFollow(true);
    }
  }, [follow]);

  // Reset follow state when startFollowing prop changes
  useEffect(() => {
    setFollow(startFollowing);
  }, [startFollowing]);

  return <>{render({ follow, onScroll: handleScroll })}</>;
};

export default ScrollFollow;