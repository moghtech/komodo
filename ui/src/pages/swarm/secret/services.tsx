import SwarmServicesSection, {
  SwarmServicesSectionProps,
} from "@/components/swarm/services-section";
import { useRead } from "@/lib/hooks";

export interface SwarmSecretServicesSectionProps extends Omit<
  SwarmServicesSectionProps,
  "services"
> {
  secret: string | undefined;
}

export default function SwarmSecretServicesSection({
  id,
  secret,
  ...props
}: SwarmSecretServicesSectionProps) {
  const services =
    useRead("ListSwarmServices", { swarm: id }).data?.filter(
      (service) => secret && service.Secrets.includes(secret),
    ) ?? [];
  return <SwarmServicesSection id={id} services={services} {...props} />;
}
